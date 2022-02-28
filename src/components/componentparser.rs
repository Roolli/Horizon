use crate::components::componentparser::ComponentParserError::NotFound;
use crate::components::modelcollider::ModelCollider;
use crate::components::physicshandle::PhysicsHandle;
use crate::components::transform::Transform;
use crate::components::ComponentStorage;
use crate::renderer::primitives::lights::pointlight::PointLight;
use crate::scripting::util::entityinfo::Component;
use crate::systems::physics::PhysicsWorld;
use crate::ECSContainer;
use rapier3d::na::{Point3, UnitQuaternion, Vector3};
use rapier3d::prelude::*;
use ref_thread_local::Ref;
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
    pub fn parse<'a>(
        &self,
        component_data: Component,
        entity: Entity,
        world: &'a World,
    ) -> Result<(), ComponentParserError> {
        self.next.parse(component_data, entity, world)
    }
}
impl Default for ComponentParser {
    fn default() -> Self {
        ComponentParser {
            next: Box::new(TransformComponentParser {
                next: Some(Box::new(PhysicsComponentParser {
                    next: Some(Box::new(PointLightComponentParser { next: None })),
                })),
            }),
        }
    }
}

pub trait ParseComponent {
    fn parse<'a>(
        &self,
        component_data: Component,
        entity: Entity,
        world: &'a World,
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
                let mass = component_data
                    .mass
                    .ok_or(ComponentParserError::InvalidData("Mass"))?;
                let mut rigid_body_handle: Option<RigidBodyHandle> = None;
                let mut collider_handle: Option<ColliderHandle> = None;
                let pos = if let Some(val) = world.read_storage::<Transform>().get(entity) {
                    val.position
                } else {
                    Vector3::new(0.0, 0.0, 0.0)
                };
                match component_data.body_type {
                    Some(crate::scripting::util::RigidBodyType::Dynamic) => {
                        let colliders = world.read_component::<ModelCollider>();
                        let collider = colliders
                            .get(world.entities().entity(model))
                            .ok_or(ComponentParserError::InvalidData("modelCollider"))?;

                        let collider = collider.0.build();
                        let rigid_body = RigidBodyBuilder::new_dynamic()
                            .position(Isometry::new(
                                Vector3::new(pos.x, pos.y, pos.z),
                                Vector3::new(0.0, 0.0, 0.0),
                            ))
                            .additional_mass(mass as f32)
                            .build();
                        let body_handle = physics_world.add_rigid_body(rigid_body);

                        collider_handle = Some(physics_world.add_collider(collider, body_handle));
                        rigid_body_handle = Some(body_handle);
                    }
                    Some(crate::scripting::util::RigidBodyType::Static) => {
                        let rigid_body = RigidBodyBuilder::new_static()
                            .position(Isometry::new(pos, Vector3::new(0.0, 0.0, 0.0)))
                            .additional_mass(mass as f32)
                            .build();
                        let body_handle = physics_world.add_rigid_body(rigid_body);
                        let scale: Vector3<f32> = if let Some(scale_vals) = component_data.scale {
                            scale_vals.into()
                        } else {
                            Vector3::new(1.0, 1.1, 1.0)
                        };
                        let collider = ColliderBuilder::cuboid(scale.x, scale.y, scale.z).build();
                        collider_handle = Some(physics_world.add_collider(collider, body_handle));
                        rigid_body_handle = Some(body_handle);
                    }
                    _ => return Err(ComponentParserError::InvalidData("bodyType")),
                }

                if let (Some(body_handle), Some(collision_handle)) =
                    (rigid_body_handle, collider_handle)
                {
                    let mut physics_storage = world.write_component::<PhysicsHandle>();
                    physics_storage
                        .insert(
                            entity,
                            PhysicsHandle {
                                collider_handle: collision_handle,
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
    fn parse<'a>(
        &self,
        component_data: Component,
        entity: Entity,
        world: &World,
    ) -> Result<(), ComponentParserError> {
        if component_data.component_type == "pointLight" {
            let pos = if let Some(val) = world.read_storage::<Transform>().get(entity) {
                val.position
            } else {
                return Err(ComponentParserError::MissingDependantComponent("Transform"));
            };
            let attenuation_values = component_data.attenuation.unwrap();
            let color = component_data.color.unwrap();
            let mut point_light_storage = world.write_component::<PointLight>();
            point_light_storage
                .insert(
                    entity,
                    PointLight::new(
                        Point3::from(pos),
                        wgpu::Color {
                            r: color.x(),
                            g: color.y(),
                            b: color.z(),
                            a: color.w(),
                        },
                        attenuation_values.x,
                        attenuation_values.y,
                        attenuation_values.z,
                        None, //TODO: add attachment
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