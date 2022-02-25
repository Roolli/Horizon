use crate::components::componentparser::ComponentParserError::NotFound;
use crate::components::transform::Transform;
use crate::components::ComponentStorage;
use crate::scripting::util::entityinfo::Component;
use crate::ECSContainer;
use rapier3d::na::UnitQuaternion;
use specs::{Builder, Entity, EntityBuilder, WorldExt};

pub trait ParseComponent<'a> {
    fn parse(
        next_component: Option<dyn ParseComponent>,
        component_data: Component,
        ecs_container: &mut ECSContainer,
        entity_builder: EntityBuilder,
    ) -> Result<Entity, ComponentParserError>;
}

impl<'a> ParseComponent<'a> for TransformComponentParser<'a> {
    fn parse(
        next_component: Option<Box<dyn ParseComponent>>,
        component_data: Component,
        ecs_container: &mut ECSContainer,
        entity_builder: EntityBuilder,
    ) -> Result<EntityBuilder<'a>, ComponentParserError> {
        if component_data.component_type.as_str() == "transform" {
            let mut model_id = None;
            if let Some(model) = component_data.model {
                let entities = ecs_container.world.entities();
                model_id = Some(entities.entity(model));
            }
            let rot = component_data.rotation.unwrap();
            let transform_val = Transform::new(
                component_data.position.unwrap().into(),
                UnitQuaternion::from_euler_angles(rot.x, rot.y, rot.z),
                component_data.scale.unwrap().into(),
                model_id,
            );
            entity_builder.with(transform_val);
            if let Some(next) = next_component.0 {
                next.parse(next, component_data, ecs_container, entity_builder)
            } else {
                Ok(entity_builder);
            }
        }
        Err(ComponentParserError::NotFound(
            component_data.component_type.to_owned(),
        ))
    }
}

pub enum ComponentParserError {
    NotFound(String), // in theory should never happen due to it being generic
    InvalidData(String),
}
