use bevy::prelude::*;
use crate::collision_world::CollisionWorld;
use crate::hex_table::HexTable;
use crate::table::Table;

#[derive(Component)]
pub struct Plate{

}
#[derive(Component)]
pub struct CollisionAutomation {
    pub world: CollisionWorld<u8>,
    pub view: Table<u8>,
    pub border: HexTable<u8>,
}

