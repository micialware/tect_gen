mod basic;
mod seed_automation;
mod plate_automation;
mod subplate_automation;

use crate::basic::{IntoImage, Table};
use bevy::asset::io::embedded::GetAssetServer;
use bevy::input::keyboard::keyboard_input_system;
use bevy::prelude::*;
use bevy::window::{ExitCondition, WindowResolution};
use rand::{Rng, RngExt};
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use seed_automation::SeedAutomation;

const RECTANGLE_SIDE: f32 = 750.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Automations".into(),
                resolution: WindowResolution::new(RECTANGLE_SIDE as u32, RECTANGLE_SIDE as u32),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .insert_resource(Time::<Fixed>::from_hz(1024.0))
        .add_systems(Update, seed_automation::update_automation)
        .add_systems(Update, seed_automation::update_automation_view)
        .add_systems(Update, plate_automation::update_automation)
        .add_systems(Update, plate_automation::update_automation_view)
        .add_systems(Update, subplate_automation::update_automation)
        .add_systems(Update, subplate_automation::update_automation_view)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2d);

    let handle = images.add(Image::default());

    let mut shape = Sprite::from_image(handle);
    shape.custom_size = Some(Vec2::new(RECTANGLE_SIDE, RECTANGLE_SIDE));

    let automation = SeedAutomation {
        world: Table::new(false, 16),
    };

    commands.spawn((shape, automation));

    let seeded_rng = ChaCha8Rng::seed_from_u64(rand::random());
    commands.insert_resource(SeededRng(seeded_rng));
}

#[derive(Resource)]
struct SeededRng(ChaCha8Rng);

#[cfg(test)]
mod tests {
    use crate::basic::Table;

    #[test]
    fn around_1(){
        let mut table = Table::new(false, 16);
        let around = table.around(0, 0);
        assert_eq!(around.len(), 2);
    }

    #[test]
    fn around_2(){
        let mut table = Table::new(false, 16);
        let around = table.around(15, 15);
        assert_eq!(around.len(), 2);
    }

    #[test]
    fn around_3(){
        let mut table = Table::new(false, 16);
        let around = table.around(5, 5);
        assert_eq!(around.len(), 4);
    }
    #[test]
    fn around_1_line(){
        let mut table = Table::new(false, 16);
        let around = table.around_line(0);
        assert_eq!(around.len(), 2);
    }

    #[test]
    fn around_2_line(){
        let mut table = Table::new(false, 16);
        let around = table.around_line(16 * 16 - 1);
        assert_eq!(around.len(), 2);
    }

    #[test]
    fn around_3_line(){
        let mut table = Table::new(false, 16);
        let around = table.around_line(24);
        assert_eq!(around.len(), 4);
    }
}