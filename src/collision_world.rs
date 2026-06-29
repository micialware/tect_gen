use bevy::prelude::*;

#[derive(Clone)]
pub struct CollisionWorld<T: Clone> {
    bodies: Vec<CollisionBody<T>>
}

#[derive(Clone)]
pub struct CollisionBody<T: Clone> {
    value: T,
    offset: Vec2,
    border: Vec<Vec2>,
    forces: Vec<Vec2>,
    resistance: f32,
}

impl<T: Clone> CollisionWorld<T> {
    pub fn new(parts: Vec<(Vec<Vec2>, T)>) -> CollisionWorld<T> {
        Self{
            bodies: parts.iter().map(|(border, id)| {CollisionBody{
                value: id.clone(),
                offset: Vec2::ZERO,
                forces: vec![Vec2::ZERO; border.len()],
                border: border.clone(),
                resistance: 1.0,
            }}).collect(),
        }
    }
}