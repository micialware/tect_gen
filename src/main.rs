mod hex_table;
mod plate_automation;
mod seed_automation;
mod subplate_automation;
mod table;

use crate::subplate_automation::HexMatrixView;
use crate::table::{IntoImage, Table};
use bevy::asset::io::embedded::GetAssetServer;
use bevy::image::TextureFormatPixelInfo;
use bevy::input::keyboard::keyboard_input_system;
use bevy::prelude::*;
use bevy::window::{ExitCondition, WindowResolution};
use rand::{Rng, RngExt};
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use seed_automation::SeedAutomation;

extern crate bevy;
const RECTANGLE_SIDE: f32 = 1500.0;
const WINDOW_SIDE: u32 = 1700;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Automations".into(),
                resolution: WindowResolution::new(WINDOW_SIDE, WINDOW_SIDE),
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
        .add_systems(Update, subplate_automation::setup_hex_matrix)
        .add_systems(Update, subplate_automation::update_hex_matrix_view)
        .add_systems(Update, moving_system)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2d);

    let handle = images.add(Image::default());

    let mut shape = Sprite::from_image(handle);
    shape.custom_size = Some(Vec2::new(RECTANGLE_SIDE, RECTANGLE_SIDE));

    let automation = SeedAutomation {
        world: Table::new(false, 64),
    };

    commands.spawn((shape, automation, Moving));

    let seeded_rng = ChaCha8Rng::seed_from_u64(rand::random());
    commands.insert_resource(SeededRng(seeded_rng));

    let mut hex_image = Image::default();
    hex_image.data = Some(vec![
        0;
        hex_image
            .texture_descriptor
            .format
            .pixel_size()
            .unwrap_or(0)
    ]);
    let handle = images.add(hex_image);

    let mut shape = Sprite::from_image(handle);
    shape.custom_size = Some(Vec2::new(RECTANGLE_SIDE, RECTANGLE_SIDE));

    commands.spawn((
        shape,
        HexMatrixView,
        Transform::from_xyz(0.0, 0.0, 1.0),
        Moving,
    ));
}

fn moving_system(mut hex: Query<&mut Transform, With<Moving>>, keys: Res<ButtonInput<KeyCode>>) {
    for mut transform in hex {
        if keys.pressed(KeyCode::ArrowDown) {
            transform.translation.y += 3.0;
        }

        if keys.pressed(KeyCode::ArrowUp) {
            transform.translation.y -= 3.0;
        }

        if keys.pressed(KeyCode::ArrowLeft) {
            transform.translation.x += 3.0;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            transform.translation.x -= 3.0;
        }

        if keys.pressed(KeyCode::PageUp) {
            transform.scale -= 0.05;
        }

        if keys.pressed(KeyCode::PageDown) {
            transform.scale += 0.05;
        }

        if keys.just_pressed(KeyCode::Home) {
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
            transform.scale = Vec3::ONE;
        }
    }
}

#[derive(Resource)]
struct SeededRng(ChaCha8Rng);

#[derive(Component)]
pub struct Moving;

#[cfg(test)]
mod tests {
    use crate::hex_table::HexTable;
    use crate::table::Table;
    use bevy::render::render_resource::encase::private::RuntimeSizedArray;

    #[test]
    fn around_1() {
        let mut table = Table::new(false, 16);
        let around = table.around(0, 0);
        assert_eq!(around.len(), 2);
    }

    #[test]
    fn around_2() {
        let mut table = Table::new(false, 16);
        let around = table.around(15, 15);
        assert_eq!(around.len(), 2);
    }

    #[test]
    fn around_3() {
        let mut table = Table::new(false, 16);
        let around = table.around(5, 5);
        assert_eq!(around.len(), 4);
    }
    #[test]
    fn around_1_line() {
        let mut table = Table::new(false, 16);
        let around = table.around_line(0);
        assert_eq!(around.len(), 2);
    }

    #[test]
    fn around_2_line() {
        let mut table = Table::new(false, 16);
        let around = table.around_line(16 * 16 - 1);
        assert_eq!(around.len(), 2);
    }

    #[test]
    fn around_3_line() {
        let mut table = Table::new(false, 16);
        let around = table.around_line(24);
        assert_eq!(around.len(), 4);
    }

    #[test]
    fn hex_table_1() {
        let mut table = HexTable::new(10, false, 4.0);
        let coord = table.get_offset_of(9, 0);
        assert_eq!(coord, (36.0, 0.0));
    }

    #[test]
    fn hex_table_around_1() {
        let mut table = HexTable::new(10, false, 4.0);
        let around = table.around(9, 0);
        assert_eq!(around.len(), 3);
    }
}
