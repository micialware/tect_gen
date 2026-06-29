use crate::subplate_automation::{HexMatrixRequest, SubPlateAutomation, ViewRedrawRequest};
use crate::table::{IntoImage, Table};
use crate::SeededRng;
use bevy::asset::Assets;
use bevy::image::Image;
use bevy::input::ButtonInput;
use bevy::prelude::{Commands, Component, Entity, KeyCode, Query, Res, ResMut, Single, Sprite};
use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;

pub fn update_automation_view(
    query: Single<(&Sprite, &PlateAutomation)>,
    request: Single<(&ViewRedrawRequest, Entity)>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {

    let (sprite, automation) = query.into_inner();
    let image = automation.world.get_image_data();

    // images.remove(sprite.image.id());
    images.insert(sprite.image.id(), image).unwrap();

    commands.entity(request.into_inner().1).despawn();
}

pub fn update_automation(
    mut query: Single<(&mut PlateAutomation, Entity)>,
    mut seeded_rng: ResMut<SeededRng>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {


    let (mut automation, entity) = query.into_inner();
    if keys.just_pressed(KeyCode::Space) || keys.all_pressed([KeyCode::Space, KeyCode::ShiftLeft]) {
        let random = &mut seeded_rng.0;

        automation.next(random);
        commands.spawn(ViewRedrawRequest);

    }

    if keys.just_pressed(KeyCode::AltLeft) {
        automation.world.grow();
        commands.spawn(ViewRedrawRequest);
        println!("Map size {}", automation.world.side);

    }

    if keys.just_pressed(KeyCode::Enter) {
        println!("Switching to SubPlateAutomation");
        let mut new_table = Table::<u8>::new(0, automation.world.side);
        automation.world.convert_copy(&mut new_table, |value| { if value { u8::MAX } else { 0 } });
        commands.entity(entity).remove::<PlateAutomation>().insert(SubPlateAutomation{
            world: new_table
        });
        commands.spawn(HexMatrixRequest);
    }
}

#[derive(Component)]
pub struct PlateAutomation {
    pub(crate) world: Table<bool>,
}

impl PlateAutomation {
    fn next(&mut self, rng: &mut ChaCha8Rng) {
        // let time = Instant::now();
        let range = 0..self.world.side * self.world.side;
        let randoms = range.clone().map(|_| rng.random::<f32>()).collect::<Vec<f32>>();
        let updated = range.into_par_iter().zip(randoms).map(|(index, random)| {
            if *self.world.get(index) {
                return true;
            }

            let around = self.world.around_line(index).iter().filter(|v| ***v).count();

            if around == 0 {
                return false;
            }

            let chance = (around as f32 * 0.25).powi(2);
            if random > chance {
                return false;
            }

            return true;
        }).collect::<Vec<bool>>();

        self.world.data = updated;

        // println!("Done in {} mcs", time.elapsed().as_micros());

    }
}
