pub mod entityinfo;
pub mod glmconversion;
pub mod horizonentity;
pub mod componentconversions;
pub mod horizonresource;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum RigidBodyType {
    Dynamic = 0,
    Kinematic = 1,
    Static = 2,
}
