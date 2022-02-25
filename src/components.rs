pub mod assetidentifier;
pub mod componentparser;
pub mod componenttypes;
pub mod modelcollider;
pub mod physicshandle;
pub mod scriptingcallback;
pub mod transform;

pub struct ComponentStorage<T> {
    inner: T,
}

impl<T> ComponentStorage<T> {
    pub fn new(inner: T) -> Self {
        ComponentStorage { inner }
    }
}
