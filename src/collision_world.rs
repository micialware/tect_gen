use bevy::prelude::*;

#[derive(Clone)]
pub struct CollisionWorld<T: Clone> {
    pub(crate) bodies: Vec<CollisionBody<T>>
}

#[derive(Clone)]
pub struct CollisionBody<T: Clone> {
    pub(crate) value: T,
    pub(crate) position: Vec2,
    pub(crate) border: Vec<Vec2>,
    pub(crate) forces: Vec<Vec2>,
    pub(crate) mass: f32,
}

impl<T: Clone> CollisionWorld<T> {
    
}