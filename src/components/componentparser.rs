use crate::components::collisionshape::CollisionShape;
use crate::components::physicshandle::PhysicsHandle;
use crate::components::transform::Transform;
use crate::renderer::model::HorizonModel;
use crate::renderer::primitives::lights::pointlight::PointLight;
use crate::renderer::primitives::mesh::{VertexAttribValues, VertexAttributeType};
use crate::scripting::util::entityinfo::Component;
use crate::systems::physics::PhysicsWorld;
use rapier3d::na::{Isometry3, Matrix4, Point3, Quaternion, UnitQuaternion, Vector3};
use rapier3d::parry::transformation::vhacd::VHACDParameters;
use rapier3d::prelude::*;
use specs::{Builder, Entity, EntityBuilder, Join, World, WorldExt};

#[derive(Debug, Clone)]
pub enum ComponentParserError {
    NotFound(String), // in theory should never happen
    InvalidData(&'static str),
    MissingDependantComponent(&'static str),
}

pub struct ComponentParser {
    next: Box<dyn ParseComponent>,
}
impl ComponentParser {
    pub fn parse(
        &self,
        component_data: Component,
        entity: Entity,
        world: &World,
    ) -> Result<(), ComponentParserError> {
        self.next.parse(component_data, entity, world)
    }
}
impl Default for ComponentParser {
    fn default() -> Self {
        ComponentParser {
            next: Box::new(TransformComponentParser {
                next: Some(Box::new(PhysicsComponentParser {
                    next: Some(Box::new(PointLightComponentParser {
                        next: Some(Box::new(CollisionShapeParser { next: None })),
                    })),
                })),
            }),
        }
    }
}

pub trait ParseComponent {
    fn parse(
        &self,
        component_data: Component,
        entity: Entity,
        world: &World,
    ) -> Result<(), ComponentParserError>;
}
pub struct TransformComponentParser {
    next: Option<Box<dyn ParseComponent>>,
}
impl ParseComponent for TransformComponentParser {
    fn parse(
        &self,
        component_data: Component,
        entity: Entity,
        world: &World,
    ) -> Result<(), ComponentParserError> {
        if component_data.component_type.as_str() == "transform" {
            let mut model_id = None;
            if let Some(model) = component_data.model {
                let entities = world.entities();
                model_id = Some(entities.entity(model));
            }
            let rot = component_data
                .rotation
                .ok_or(ComponentParserError::InvalidData("rotation"))?;
            let transform_val = Transform::new(
                component_data
                    .position
                    .ok_or(ComponentParserError::InvalidData("position"))?
                    .into(),
                UnitQuaternion::from_euler_angles(rot.x, rot.y, rot.z),
                component_data
                    .scale
                    .ok_or(ComponentParserError::InvalidData("scale"))?
                    .into(),
                model_id,
            );
            let mut transform_storage = world.write_storage::<Transform>();
            transform_storage.insert(entity, transform_val).unwrap();

            Ok(())
        } else if let Some(ref next) = self.next {
            next.parse(component_data, entity, world)
        } else {
            Err(ComponentParserError::NotFound(
                component_data.component_type,
            ))
        }
    }
}
pub struct PhysicsComponentParser {
    next: Option<Box<dyn ParseComponent>>,
}

impl ParseComponent for PhysicsComponentParser {
    fn parse(
        &self,
        component_data: Component,
        entity: Entity,
        world: &World,
    ) -> Result<(), ComponentParserError> {
        if component_data.component_type.as_str() == "physics" {
            // Only add physics to valid already created objects
            if let Some(model) = component_data.model {
                let mut physics_world = world.write_resource::<PhysicsWorld>();
                let mass = component_data.mass.unwrap_or_default();
                let model_data = world.read_component::<HorizonModel>();
                let model = model_data
                    .get(world.entities().entity(model))
                    .ok_or(ComponentParserError::InvalidData("model"))?;

                let mut rigid_body_handle: Option<RigidBodyHandle> = None;
                let mut collider_handles: Vec<ColliderHandle> = Vec::new();

                let transform_storage = world.read_storage::<Transform>();
                let transform = transform_storage
                    .get(entity)
                    .ok_or(ComponentParserError::MissingDependantComponent("Transform"))?;
                match component_data.body_type {
                    Some(crate::scripting::util::RigidBodyType::Dynamic) => {
                        let mut rigid_body_builder = RigidBodyBuilder::new_dynamic()
                            .position(Isometry3::new(
                                Vector3::new(
                                    transform.position.x,
                                    transform.position.y,
                                    transform.position.z,
                                ),
                                transform.rotation.scaled_axis(),
                            ))
                            .user_data(entity.id() as u128)
                            .additional_mass(mass as f32);
                        if let Some(damping_values) = component_data.damping {
                            for damping in damping_values {
                                match damping.damping_type.as_str() {
                                    "linear" => {
                                        rigid_body_builder =
                                            rigid_body_builder.linear_damping(damping.amount);
                                    }
                                    "angular" => {
                                        rigid_body_builder =
                                            rigid_body_builder.angular_damping(damping.amount);
                                    }
                                    _ => return Err(ComponentParserError::InvalidData("damping")),
                                }
                            }
                        }
                        let rigid_body = rigid_body_builder.build();
                        let body_handle = physics_world.add_rigid_body(rigid_body);
                        let mut convex_decs = Vec::new();
                        for mesh in &model.meshes {
                            for primitive in &mesh.primitives {
                                if let Some(VertexAttribValues::Float32x3 { 0: values }) =
                                    primitive.mesh.attribute(VertexAttributeType::Position)
                                {
                                    let vertices: Vec<Point3<f32>> = primitive
                                        .mesh
                                        .indices
                                        .as_ref()
                                        .unwrap()
                                        .iter()
                                        .map(|v| {
                                            let vertex = values[*v as usize];
                                            Point3::new(vertex[0], vertex[1], vertex[2])
                                        })
                                        .collect::<Vec<_>>();
                                    if let Some(builder) = ColliderBuilder::convex_hull(&vertices) {
                                        convex_decs.push(builder.build());
                                    }
                                }
                            }
                        }
                        let compound_collider = ColliderBuilder::compound(
                            convex_decs
                                .into_iter()
                                .map(|v| (Isometry::identity(), v.shared_shape().clone()))
                                .collect::<Vec<_>>(),
                        )
                        .active_events(
                            ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS,
                        )
                        .build();
                        collider_handles
                            .push(physics_world.add_collider(compound_collider, body_handle));
                        rigid_body_handle = Some(body_handle);
                    }
                    // use tri-mesh for static rigid Bodies.
                    Some(crate::scripting::util::RigidBodyType::Static) => {
                        let rigid_body = RigidBodyBuilder::new_kinematic_velocity_based()
                            .position(Isometry3::new(
                                Vector3::new(
                                    transform.position.x,
                                    transform.position.y,
                                    transform.position.z,
                                ),
                                transform.rotation.scaled_axis(),
                            ))
                            .user_data(entity.id() as u128)
                            .additional_mass(mass as f32)
                            .build();
                        let body_handle = physics_world.add_rigid_body(rigid_body);

                        let mut triangles_meshes = Vec::new();

                        for mesh in &model.meshes {
                            for primitive in &mesh.primitives {
                                if let Some(VertexAttribValues::Float32x3 { 0: values }) =
                                    primitive.mesh.attribute(VertexAttributeType::Position)
                                {
                                    let vertices: Vec<Point3<f32>> = primitive
                                        .mesh
                                        .indices
                                        .as_ref()
                                        .unwrap()
                                        .iter()
                                        .map(|v| {
                                            let vertex = values[*v as usize];
                                            Point3::new(vertex[0], vertex[1], vertex[2])
                                        })
                                        .collect::<Vec<_>>();
                                    if let Some(collider_builder) =
                                        ColliderBuilder::convex_hull(&vertices)
                                    {
                                        triangles_meshes.push(collider_builder.build());
                                    }
                                }
                            }
                        }

                        let compound_collider = ColliderBuilder::compound(
                            triangles_meshes
                                .into_iter()
                                .map(|v| (Isometry::identity(), v.shared_shape().clone()))
                                .collect::<Vec<_>>(),
                        )
                        .active_events(
                            ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS,
                        )
                        .build();
                        collider_handles
                            .push(physics_world.add_collider(compound_collider, body_handle));
                        rigid_body_handle = Some(body_handle);
                    }
                    _ => return Err(ComponentParserError::InvalidData("bodyType")),
                }

                if let Some(body_handle) = rigid_body_handle {
                    let mut physics_storage = world.write_component::<PhysicsHandle>();
                    physics_storage
                        .insert(
                            entity,
                            PhysicsHandle {
                                collider_handles,
                                rigid_body_handle: body_handle,
                            },
                        )
                        .unwrap();
                    Ok(())
                } else {
                    Err(ComponentParserError::InvalidData(
                        "bodyhandle or collision handle",
                    ))
                }
            } else {
                Err(ComponentParserError::MissingDependantComponent("transform"))
            }
        } else if let Some(ref next) = self.next {
            next.parse(component_data, entity, world)
        } else {
            Err(ComponentParserError::NotFound(
                component_data.component_type,
            ))
        }
    }
}
pub struct PointLightComponentParser {
    next: Option<Box<dyn ParseComponent>>,
}
impl ParseComponent for PointLightComponentParser {
    fn parse(
        &self,
        component_data: Component,
        entity: Entity,
        world: &World,
    ) -> Result<(), ComponentParserError> {
        if component_data.component_type == "pointLight" {
            let radius = component_data.radius.unwrap();
            let color = component_data.color.unwrap();
            let mut point_light_storage = world.write_component::<PointLight>();

            let attached_entity = component_data
                .attached_to
                .map(|val| world.entities().entity(val));
            point_light_storage
                .insert(
                    entity,
                    PointLight::new(
                        Vector3::new(color.x as f32, color.y as f32, color.z as f32),
                        radius,
                        attached_entity,
                    ),
                )
                .unwrap();
            Ok(())
        } else if let Some(ref next) = self.next {
            next.parse(component_data, entity, world)
        } else {
            Err(ComponentParserError::NotFound(
                component_data.component_type,
            ))
        }
    }
}
pub struct CollisionShapeParser {
    next: Option<Box<dyn ParseComponent>>,
}
impl ParseComponent for CollisionShapeParser {
    fn parse(
        &self,
        component_data: Component,
        entity: Entity,
        world: &World,
    ) -> Result<(), ComponentParserError> {
        if component_data.component_type == "CollisionShape" {
            let transform = if let Some(val) = world.read_storage::<Transform>().get(entity) {
                *val
            } else {
                return Err(ComponentParserError::MissingDependantComponent("Transform"));
            };
            if let Some(collider_info) = component_data.collision_shape {
                let shape = match collider_info
                    .collision_shape_type
                    .unwrap_or_default()
                    .as_str()
                {
                    "Cuboid" => {
                        let half_extents = if let Some(extents) = collider_info.half_extents {
                            extents
                        } else {
                            return Err(ComponentParserError::InvalidData("halfExtents"));
                        };
                        SharedShape::new(Cuboid::new(half_extents.into()))
                    }
                    "Ball" => {
                        let radius = if let Some(radius) = collider_info.radius {
                            radius
                        } else {
                            return Err(ComponentParserError::InvalidData("Radius"));
                        };
                        SharedShape::new(Ball::new(radius))
                    }
                    "Capsule" => {
                        if let Some(half_height) = collider_info.half_height {
                            let radius = if let Some(radius) = collider_info.radius {
                                radius
                            } else {
                                return Err(ComponentParserError::InvalidData("Radius"));
                            };
                            let shape = match collider_info
                                .capsule_type
                                .unwrap_or_default()
                                .as_str()
                            {
                                "x" => Capsule::new_x(half_height, radius),
                                "y" => Capsule::new_y(half_height, radius),
                                "z" => Capsule::new_z(half_height, radius),
                                _ => return Err(ComponentParserError::InvalidData("capsuleType")),
                            };
                            SharedShape::new(shape)
                        } else {
                            return Err(ComponentParserError::InvalidData("HalfHeight"));
                        }
                    }
                    _ => return Err(ComponentParserError::InvalidData("CollisionShape")),
                };

                let collision = ColliderBuilder::new(shape)
                    .position(Isometry3::new(
                        transform.position,
                        transform.rotation.scaled_axis(),
                    ))
                    .active_events(ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS)
                    .user_data(entity.id() as u128)
                    .sensor(true)
                    .build();

                let mut physics_world = world.write_resource::<PhysicsWorld>();
                let collider_handle = physics_world.collider_set.insert(collision);
                let mut collision_shape_storage = world.write_storage::<CollisionShape>();
                collision_shape_storage
                    .insert(
                        entity,
                        CollisionShape {
                            collider: collider_handle,
                        },
                    )
                    .unwrap();
                Ok(())
            } else {
                Err(ComponentParserError::InvalidData("CollisionShape"))
            }
        } else if let Some(ref next) = self.next {
            next.parse(component_data, entity, world)
        } else {
            Err(ComponentParserError::NotFound(
                component_data.component_type,
            ))
        }
    }
}
