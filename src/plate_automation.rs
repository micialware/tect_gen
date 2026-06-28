use rayon::iter::IndexedParallelIterator;
use std::time::Instant;
use crate::SeededRng;
use crate::table::{IntoImage, Table};
use bevy::asset::{Assets, RenderAssetUsages};
use bevy::image::{BevyDefault, Image};
use bevy::input::ButtonInput;
use bevy::prelude::{Color, Commands, Component, Entity, KeyCode, Query, Res, ResMut, Sprite};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::IntoParallelIterator;
use crate::seed_automation::SeedAutomation;
use crate::subplate_automation::{HexMatrixRequest, SubPlateAutomation};
use rayon::iter::ParallelIterator;

pub fn update_automation_view(
    query: Query<(&Sprite, &PlateAutomation)>,
    mut images: ResMut<Assets<Image>>,
) {
    if query.is_empty() {
        return;
    }
    let (sprite, automation) = query.iter().next().unwrap();
    let size = automation.world.side as u32;
    let data = automation.world.get_image_data();
    let image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::bevy_default(),
        RenderAssetUsages::default(),
    );

    images.remove(sprite.image.id());
    images.insert(sprite.image.id(), image).unwrap();
}

pub fn update_automation(
    mut query: Query<(&mut PlateAutomation, Entity)>,
    mut seeded_rng: ResMut<SeededRng>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    if query.is_empty() {
        return;
    }

    let (mut automation, entity) = query.iter_mut().next().unwrap();
    if keys.just_pressed(KeyCode::Space) || keys.all_pressed([KeyCode::Space, KeyCode::ShiftLeft]) {
        let random = &mut seeded_rng.0;

        automation.next(random)
    }

    if keys.just_pressed(KeyCode::AltLeft) {
        automation.world.grow();
        println!("Map size {}", automation.world.side)
    }

    if keys.just_pressed(KeyCode::Enter) {
        println!("Switching to SubPlateAutomation");
        let mut new_table = Table::<u8>::new(0, automation.world.side);
        automation.world.convert_copy(&mut new_table, |value| { if value { u8::MAX } else { 0 } });
        commands.entity(entity).remove::<PlateAutomation>().insert(SubPlateAutomation{
            world: new_table
        });
        commands.spawn((HexMatrixRequest));
    }
}

#[derive(Component)]
pub struct PlateAutomation {
    pub(crate) world: Table<bool>,
}

impl PlateAutomation {
    fn next(&mut self, rng: &mut ChaCha8Rng) {
        let time = Instant::now();
        let range = 0..self.world.side * self.world.side;
        let randoms = range.clone().map(|x| rng.random::<f32>()).collect::<Vec<f32>>();
        let updated = range.into_par_iter().zip(randoms).map(|(index, random)| {
            if *self.world.get(index) {
                return true;
            }

            let around = self.world.around_line(index).iter().filter(|v| ***v).count();

            if around == 0 {
                return false;
            }

            let chance = (around as f32 * 0.2).powi(2);
            if random > chance {
                return false;
            }

            return true;
        }).collect::<Vec<bool>>();

        self.world.data = updated;

    }
}
