use crate::table::{IntoImage, Table};
use crate::hex_table::HexTable;
use crate::SeededRng;
use bevy::asset::{Assets, RenderAssetUsages};
use bevy::image::{BevyDefault, Image};
use bevy::input::ButtonInput;
use bevy::prelude::{Color, Commands, Component, Entity, KeyCode, Query, Res, ResMut, Sprite, Text, With};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
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
    mut hex_query: Query<&HexMatrixBuild>,
    mut seeded_rng: ResMut<SeededRng>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
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
        automation.calculate_front_lines(&mut commands, hex_query);
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

            self.world
                .set(index, rng.random::<u8>());
            break;
        }
    }

    fn calculate_front_lines(&self, commands: &mut Commands, hex_query: Query<&HexMatrixBuild>) {

        let generator = &hex_query.iter().next().unwrap().hex_matrix;
        let all_points = generator.calculate();

        let own_points = all_points.iter().map(|(x, y)| {
            let x_table = *x as usize;
            let y_table = *y as usize;

            (*self.world.get_dim(x_table, y_table), x, y)
        }).filter(|(v, _, _)| {
            *v != 0
        });

        commands.spawn((HexMatrixRedrawRequest));
    }
    fn calculate_subplates(&self, commands: &mut Commands) {
        let mut list : HashMap<u8, Vec<usize>> = HashMap::new();
        self.world.iter().zip(0..self.world.data.len()).for_each(|(world, index)| {
            if list.contains_key(world) {
                list.get_mut(world).unwrap().push(index);
            }
            else {
                list.insert(world.clone(), vec![index]);
            }
        });
        println!("{:?}", list.iter().map(|(c, v)| (c.clone(), v.len())).collect::<Vec<_>>());
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

    let resolution = automation.world.side / 4;

    let generator = HexTable::new(resolution, true, 4.0);
    let mut table = Table::new(false, automation.world.side);
    generator.calculate().iter().for_each(|(x, y)| {
        let x = *x as usize;
        let y = *y as usize;

        if *automation.world.get_dim(x, y) == 0 {
            return;
        }

        table.set_dim(x, y, true);
    });

    let data = table.get_image_data();
    let image = Image::new(
        Extent3d {
            width: table.side as u32,
            height: table.side as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::bevy_default(),
        RenderAssetUsages::default(),
    );

    let sprite = view_query.iter().next().unwrap();


    images.remove(sprite.image.id());
    images.insert(sprite.image.id(), image).unwrap();
    commands.spawn((HexMatrixBuild{
        points: table,
        hex_matrix: generator,
    }));
}


pub fn update_hex_matrix_view(
    mut data_query: Query<&HexMatrixBuild>,
    mut request_query: Query<Entity, With<HexMatrixRedrawRequest>>,
    mut view_query: Query<&Sprite, With<HexMatrixView>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
){
    if request_query.is_empty() {
        return;
    }

    let req = request_query.iter_mut().next().unwrap();
    commands.entity(req).despawn();
    println!("Update hex matrix view");
    let table = data_query.iter_mut().next().unwrap();
    let data = table.points.get_image_data();

    let image = Image::new(
        Extent3d {
            width: table.points.side as u32,
            height: table.points.side as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::bevy_default(),
        RenderAssetUsages::default(),
    );

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
pub struct HexMatrixBuild{
    points: Table<bool>,
    hex_matrix: HexTable<bool>
}