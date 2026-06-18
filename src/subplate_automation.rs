use rayon::iter::ParallelIterator;
use std::time::Instant;
use crate::SeededRng;
use crate::basic::{IntoImage, Table};
use bevy::asset::{Assets, RenderAssetUsages};
use bevy::image::{BevyDefault, Image};
use bevy::input::ButtonInput;
use bevy::prelude::{Color, Commands, Component, Entity, KeyCode, Query, Res, ResMut, Sprite};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::tasks::futures_lite::StreamExt;
use rand::{ RngExt};
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

pub fn update_automation_view(
    query: Query<(&Sprite, &SubPlateAutomation)>,
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
    mut query: Query<(&mut SubPlateAutomation, Entity)>,
    mut seeded_rng: ResMut<SeededRng>,
    keys: Res<ButtonInput<KeyCode>>,
    commands: Commands,

) {
    if query.is_empty() {
        return;
    }

    let (mut automation, _entity) = query.iter_mut().next().unwrap();
    if keys.just_pressed(KeyCode::Space) || keys.all_pressed([KeyCode::Space, KeyCode::ShiftLeft]) {
        let random = &mut seeded_rng.0;

        automation.next(random)
    }

    if keys.just_pressed(KeyCode::AltLeft) {
        let random = &mut seeded_rng.0;

        automation.seed(random);
    }

    if keys.just_pressed(KeyCode::AltRight) {
        automation.smooth();
    }
}

#[derive(Component)]
pub struct SubPlateAutomation {
    pub(crate) world: Table<Color>,
}

impl SubPlateAutomation {
    fn next(&mut self, rng: &mut ChaCha8Rng) {
        let time = Instant::now();

        for index in 0..self.world.side * self.world.side {
            let current_color = *self.world.get(index);
            if current_color == Color::BLACK || current_color != Color::WHITE {
                continue;
            }

            if rng.random::<f32>() < 0.7 {
                continue;
            }

            let around = self.world.around_line(index).iter().filter(|v| ***v != Color::WHITE && ***v != Color::BLACK).map(|color| (*color).clone()).collect::<Vec<_>>();

            if around.len() == 0 {
                continue;
            }
            let color = around[rng.random_range(0..around.len())];

            self.world.set(index, color);
        }

        println!("Time elapsed: {:?} ms", time.elapsed().as_millis());

    }

    fn smooth(&mut self) {
        let range = 0..self.world.side * self.world.side;
        let update = range.into_iter().map(|index| {
            let around = self.world.around_line(index);
            let mut counts: Vec<(Color, u8)> = vec![];
            for p in around {
                for count_index in 0..counts.len() {
                    if counts[count_index].0 == *p {
                        counts[count_index].1 += 1;
                        continue
                    }
                }
                counts.push((p.clone(), 1));
            }
            counts.iter().max_by_key(|(_, c)| {c}).unwrap().0
        }).collect::<Vec<Color>>();
        self.world.data = update;
    }

    fn seed(&mut self, rng: &mut ChaCha8Rng){
        let len = (self.world.side * self.world.side) as f32;
        loop {
            let index = (rng.random::<f32>() * len) as usize;
            if *self.world.get(index) == Color::BLACK {
                continue;
            }

            self.world.set(index, Color::hsv(rng.random::<f32>() * 360.0, 1.0, 1.0));
            break;
        }
    }
}
