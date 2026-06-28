use crate::hex_table::HexTable;
use crate::table::{IntoImage, Table};
use crate::SeededRng;
use bevy::asset::Assets;
use bevy::image::Image;
use bevy::input::ButtonInput;
use bevy::prelude::{
    Commands, Component, Entity, KeyCode, Query, Res, ResMut, Single, Sprite,
    With,
};
use bevy::tasks::futures_lite::StreamExt;
use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;
use std::time::Instant;

pub fn update_automation_view(
    query: Query<(&Sprite, &SubPlateAutomation)>,
    mut images: ResMut<Assets<Image>>,
) {
    if query.is_empty() {
        return;
    }
    let (sprite, automation) = query.iter().next().unwrap();
    let image = automation.world.get_image_data();

    images.remove(sprite.image.id());
    images.insert(sprite.image.id(), image).unwrap();
}

pub fn update_automation(
    query: Single<(&mut SubPlateAutomation, Entity)>,
    hex_query: Single<&mut HexMatrixBuild>,
    mut seeded_rng: ResMut<SeededRng>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    let (mut automation, _entity) = query.into_inner();
    if keys.just_pressed(KeyCode::Space) || keys.all_pressed([KeyCode::Space, KeyCode::ShiftLeft]) {
        let random = &mut seeded_rng.0;

        automation.next(random)
    }

    if keys.just_pressed(KeyCode::AltLeft) {
        let random = &mut seeded_rng.0;

        for _ in 0..16 {
            automation.seed(random);
        }
    }

    // if keys.just_pressed(KeyCode::AltRight) {
    //     automation.smooth();
    // }

    if keys.just_pressed(KeyCode::KeyS) {
        automation.calculate_subplates(&mut commands);
    }

    if keys.just_pressed(KeyCode::KeyF) {
        automation.calculate_front_lines(&mut commands, hex_query, &automation);
    }
}

#[derive(Component)]
pub struct SubPlateAutomation {
    pub(crate) world: Table<u8>,
}

impl SubPlateAutomation {
    fn next(&mut self, rng: &mut ChaCha8Rng) {
        let time = Instant::now();

        for index in 0..self.world.side * self.world.side {
            let current_color = *self.world.get(index);
            if current_color == 0 || current_color != u8::MAX {
                continue;
            }

            if rng.random::<f32>() < 0.8 {
                continue;
            }

            let around = self
                .world
                .around_line(index)
                .iter()
                .filter(|v| ***v != u8::MAX && ***v != 0)
                .map(|color| (*color).clone())
                .collect::<Vec<_>>();

            if around.len() == 0 {
                continue;
            }
            let color = around[rng.random_range(0..around.len())];

            self.world.set(index, color);
        }
    }

    fn smooth(&mut self) {
        let range = 0..self.world.side * self.world.side;
        for index in range {
            let around = self.world.around_line(index);
            let mut counts: Vec<(u8, u8)> = vec![];
            for p in around {
                for count_index in 0..counts.len() {
                    if counts[count_index].0 == *p {
                        counts[count_index].1 += 1;
                        continue;
                    }
                }
                counts.push((p.clone(), 1));
            }
            self.world
                .set(index, counts.iter().max_by_key(|(_, c)| c).unwrap().0);
        }
    }

    fn seed(&mut self, rng: &mut ChaCha8Rng) {
        let len = (self.world.side * self.world.side) as f32;
        loop {
            let index = (rng.random::<f32>() * len) as usize;
            if *self.world.get(index) == 0 {
                continue;
            }

            self.world.set(index, rng.random::<u8>());
            break;
        }
    }

    fn calculate_front_lines(
        &self,
        commands: &mut Commands,
        mut hex_query: Single<&mut HexMatrixBuild>,
        automation: &SubPlateAutomation,
    ) {
        let table = &mut hex_query.hex_matrix;
        let mut table_copy = table.clone();

        let mut edge = (0_usize, 0_usize);
        for x in 0..table.dimensions.0 {
            for y in 0..table.dimensions.1 {
                let color = *table.get_on_square_table(x, y, &automation.world);
                table.set_dim(x, y, color);
            }
        }

        for x in 0..table.dimensions.0 {
            for y in 0..table.dimensions.1 {
                let color = table.get_dim(x, y);
                if *color == 0 { continue; }
                let around = table.around(x, y);
                if around.iter().all(|x1| *x1 == around[0]) {
                    table_copy.set_dim(x, y, 0);
                }else {
                    let new_color = (color).overflowing_add(128_u8).0;

                    table_copy.set_dim(x, y, new_color);

                }
            }
        }
        hex_query.hex_matrix = table_copy;

        commands.spawn((HexMatrixRedrawRequest));
    }
    fn calculate_subplates(&self, commands: &mut Commands) {
        let mut list: HashMap<u8, Vec<usize>> = HashMap::new();
        self.world
            .iter()
            .zip(0..self.world.data.len())
            .for_each(|(world, index)| {
                if list.contains_key(world) {
                    list.get_mut(world).unwrap().push(index);
                } else {
                    list.insert(world.clone(), vec![index]);
                }
            });
        println!(
            "{:?}",
            list.iter()
                .map(|(c, v)| (c.clone(), v.len()))
                .collect::<Vec<_>>()
        );
    }
}

pub fn setup_hex_matrix(
    mut request_query: Query<(&HexMatrixRequest, Entity)>,
    mut view_query: Query<&Sprite, With<HexMatrixView>>,
    mut automation_query: Query<&SubPlateAutomation>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    if request_query.is_empty() {
        return;
    }

    let entity = request_query.iter().next().unwrap().1;
    let automation = automation_query.iter_mut().next().unwrap();
    commands.entity(entity).despawn();

    println!("Seed hex matrix");

    let resolution = automation.world.side / 2;

    let hex_table = HexTable::new(resolution, 0, 2.0);

    let image = hex_table.get_image_data();

    let sprite = view_query.iter().next().unwrap();

    images.remove(sprite.image.id());
    images.insert(sprite.image.id(), image).unwrap();
    commands.spawn(
        (HexMatrixBuild {
            hex_matrix: hex_table,
        }),
    );
}

pub fn update_hex_matrix_view(
    mut data_query: Single<&HexMatrixBuild>,
    mut request_query: Query<Entity, With<HexMatrixRedrawRequest>>,
    mut view_query: Query<&Sprite, With<HexMatrixView>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    if request_query.is_empty() {
        return;
    }

    let req = request_query.iter_mut().next().unwrap();
    commands.entity(req).despawn();
    println!("Update hex matrix view");
    let table = data_query.into_inner();
    let image = table.hex_matrix.get_image_data();

    let sprite = view_query.iter().next().unwrap();

    images.remove(sprite.image.id());
    images.insert(sprite.image.id(), image).unwrap();
}
#[derive(Component)]
pub struct HexMatrixRequest;

#[derive(Component)]
pub struct HexMatrixRedrawRequest;

#[derive(Component)]
pub struct HexMatrixView;

#[derive(Component)]
pub struct HexMatrixBuild {
    hex_matrix: HexTable<u8>,
}
