use crate::collision_world::{CollisionBody, CollisionWorld};
use crate::hex_table::HexTable;
use crate::table::{IntoImage, Table};
use crate::{Moving, SeededRng};
use bevy::asset::{Assets, RenderAssetUsages};
use bevy::image::Image;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;
use std::time::Instant;

pub fn update_automation_view(
    query: Single<(&Sprite, &SubPlateAutomation)>,
    request: Single<(&ViewRedrawRequest, Entity)>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    let (sprite, automation) = query.into_inner();
    let image = automation.world.get_image_data();

    images.insert(sprite.image.id(), image).unwrap();

    commands.entity(request.into_inner().1).despawn();
}

pub fn update_automation(
    query: Single<(&mut SubPlateAutomation, Entity)>,
    mut hex_query: Single<&mut HexMatrixBuild>,
    mut seeded_rng: ResMut<SeededRng>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let (mut automation, entity) = query.into_inner();
    if keys.just_pressed(KeyCode::Space) || keys.all_pressed([KeyCode::Space, KeyCode::ShiftLeft]) {
        let random = &mut seeded_rng.0;

        automation.next(random);
        commands.spawn(ViewRedrawRequest);
    }

    if keys.just_pressed(KeyCode::AltLeft) {
        let random = &mut seeded_rng.0;

        for i in 1..=16 {
            automation.seed(random, i * 8);
        }

        commands.spawn(ViewRedrawRequest);
    }

    if keys.just_pressed(KeyCode::AltRight) {
        automation.smooth();
        commands.spawn(ViewRedrawRequest);
    }

    if keys.just_pressed(KeyCode::KeyS) {
        automation.calculate_subplates();
    }

    if keys.just_pressed(KeyCode::KeyF) {
        automation.calculate_front_lines(&mut commands, &mut hex_query, &automation);
    }

    if keys.just_pressed(KeyCode::Enter) {
        // commands.entity(entity).despawn();
        create_collision_world(commands, &automation, &hex_query, images);
    }
}

fn create_collision_world(
    mut commands: Commands,
    query: &Mut<SubPlateAutomation>,
    hex_query: &Single<&mut HexMatrixBuild>,
    mut images: ResMut<Assets<Image>>,
) {
    println!("Switching to CollisionAutomation");
    let mut colors = query.world.data.clone();
    colors.sort();
    colors.dedup();
    colors.remove(0);
    let mut territories = vec![Vec::<(usize, usize)>::new(); colors.len()];
    let mut borders = vec![Vec::<Vec2>::new(); colors.len()];
    for x in 0..query.world.side {
        for y in 0..query.world.side {
            let color = *query.world.get_dim(x, y);
            if color == 0 {
                continue;
            }

            let index = colors.iter().position(|v| *v == color).unwrap();
            territories[index].push((x, y));
        }
    }

    for x in 0..hex_query.hex_matrix.dimensions.0 {
        for y in 0..hex_query.hex_matrix.dimensions.1 {
            let color = *query.world.get_dim(x, y);
            if color == 0 {
                continue;
            }

            let index = colors.iter().position(|v| *v == color).unwrap();
            let coordinates = hex_query.hex_matrix.get_offset_of(x, y);
            borders[index].push(Vec2::new(coordinates.0, coordinates.1));
        }
    }
    let plates_data = territories
        .iter()
        .map(|v| {
            let center = v
                .iter()
                .map(|(x, y)| Vec2::new(*x as f32, *y as f32))
                .collect::<Vec<_>>()
                .into_iter()
                .sum::<Vec2>()
                / v.len() as f32;

            let xs = v.iter().map(|(x, y)| *x).collect::<Vec<_>>();
            let ys = v.iter().map(|(x, y)| *y).collect::<Vec<_>>();
            let max_x = *xs.iter().max().unwrap();
            let max_y = *ys.iter().max().unwrap();
            let min_x = *xs.iter().min().unwrap();
            let min_y = *ys.iter().min().unwrap();
            let mut image = Image::new_fill(
                Extent3d {
                    width: (max_x - min_x) as u32 + 1,
                    height: (max_y - min_y) as u32 + 1,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                &[0, 0, 0, 0],
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            );

            let size = image.size();

            v.into_iter().for_each(|(x, y)| {
                let position = ((*x - min_x) as u32, (*y - min_y) as u32);

                image
                    .set_color_at(position.0, position.1, Color::WHITE)
                    .unwrap();
            });

            let image_offset =  Vec2::new(min_x as f32, min_y as f32);

            (v.len(), image_offset, image, Vec2::ZERO)
        })
        .collect::<Vec<_>>();

    let mut world = CollisionWorld { bodies: vec![] };

    let mut parts = Vec::with_capacity(plates_data.len());
    for index in 0..plates_data.len() {
        let (mass, center, image, offset) = plates_data.get(index).unwrap();
        let image_handle = images.add(image.clone());

        let view = commands
            .spawn((
                Transform::from_xyz(offset.x, offset.y, 0.0),
                Sprite::from_image(image_handle),
            ))
            .id();

        let size = image.size();

        let mut entity = commands.spawn((Transform::from_xyz(center.x + size.x as f32 / 2.0, -center.y - size.y as f32 / 2.0, 0.0),));
        entity.add_child(view);
        let entity_id = entity.id();

        parts.push(entity_id);

        let col_body = CollisionBody {
            value: colors[index],
            position: *center,
            border: borders[index].clone(),
            forces: vec![],
            mass: *mass as f32,
        };

        world.bodies.push(col_body);
    }
    commands.spawn((Transform::default(), Moving)).add_children(parts.as_slice());
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

        println!("Done in {} mcs", time.elapsed().as_micros());
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

    fn seed(&mut self, rng: &mut ChaCha8Rng, value: u8) {
        let len = (self.world.side * self.world.side) as f32;
        loop {
            let index = (rng.random::<f32>() * len) as usize;
            if *self.world.get(index) == 0 {
                continue;
            }

            self.world.set(index, value);
            break;
        }
    }

    fn calculate_front_lines(
        &self,
        commands: &mut Commands,
        mut hex_query: &mut Single<&mut HexMatrixBuild>,
        automation: &SubPlateAutomation,
    ) {
        let table = &mut hex_query.hex_matrix;
        let mut table_copy = table.clone();

        for x in 0..table.dimensions.0 {
            for y in 0..table.dimensions.1 {
                let color = *table.get_on_square_table(x, y, &automation.world);
                table.set_dim(x, y, color);
            }
        }

        for x in 0..table.dimensions.0 {
            for y in 0..table.dimensions.1 {
                let color = table.get_dim(x, y);
                if *color == 0 {
                    continue;
                }
                let around = table.around(x, y);
                if around.iter().all(|x1| *x1 == around[0]) {
                    table_copy.set_dim(x, y, 0);
                } else {
                    table_copy.set_dim(x, y, *color);
                }
            }
        }
        hex_query.hex_matrix = table_copy;

        commands.spawn(HexMatrixRedrawRequest);
    }
    fn calculate_subplates(&self) {
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
    request_query: Single<(&HexMatrixRequest, Entity)>,
    view_query: Single<&Sprite, With<HexMatrixView>>,
    mut automation_query: Single<&SubPlateAutomation>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let entity = request_query.into_inner().1;
    let automation = automation_query.into_inner();
    commands.entity(entity).despawn();

    println!("Seed hex matrix");

    let resolution = automation.world.side / 4;

    let hex_table = HexTable::new(resolution, 0, 4.0);

    let image = hex_table.get_image_data();

    let sprite = view_query.into_inner();

    images.insert(sprite.image.id(), image).unwrap();
    commands.spawn(HexMatrixBuild {
        hex_matrix: hex_table,
    });
}

pub fn update_hex_matrix_view(
    data_query: Single<&HexMatrixBuild>,
    request_query: Single<Entity, With<HexMatrixRedrawRequest>>,
    view_query: Single<&Sprite, With<HexMatrixView>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let req = request_query.into_inner();
    commands.entity(req).despawn();
    println!("Update hex matrix view");
    let table = data_query.into_inner();
    let image = table.hex_matrix.get_image_data();

    let sprite = view_query.into_inner();

    images.insert(sprite.image.id(), image).unwrap();
}
#[derive(Component)]
pub struct HexMatrixRequest;

#[derive(Component)]
pub struct HexMatrixRedrawRequest;

#[derive(Component)]
pub struct ViewRedrawRequest;

#[derive(Component)]
pub struct HexMatrixView;

#[derive(Component)]
pub struct HexMatrixBuild {
    hex_matrix: HexTable<u8>,
}
