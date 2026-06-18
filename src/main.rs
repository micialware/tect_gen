mod basic;
mod seed_automation;
mod plate_automation;

use crate::basic::{IntoImage, Table};
use bevy::asset::io::embedded::GetAssetServer;
use bevy::input::keyboard::keyboard_input_system;
use bevy::prelude::*;
use rand::{Rng, RngExt};
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use seed_automation::SeedAutomation;

const RECTANGLE_SIDE: f32 = 500.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .insert_resource(Time::<Fixed>::from_hz(1024.0))
        // .add_systems(Update, automation_keyboard_input_system)
        .add_systems(Update, seed_automation::update_automation_seed)
        .add_systems(Update, seed_automation::update_automation_view_seed)
        .add_systems(Update, plate_automation::update_automation_plate)
        .add_systems(Update, plate_automation::update_automation_view_plate)
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

// fn automation_keyboard_input_system(keys: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
//     if keys.just_pressed(KeyCode::Space) {
//         commands.spawn((AutomationNext,));
//     }
//
//     if keys.just_pressed(KeyCode::Tab) {
//         commands.spawn((AutomationMigrate,));
//     }
//
//     if keys.just_pressed(KeyCode::AltLeft) {
//         commands.spawn((AutomationScale,));
//     }
// }
//
// #[derive(Component)]
// struct AutomationNext;
//
// #[derive(Component)]
// struct AutomationScale;
//
// #[derive(Component)]
// struct AutomationMigrate;
#[derive(Resource)]
struct SeededRng(ChaCha8Rng);
