use crate::plate_automation::PlateAutomation;
use crate::table::{IntoImage, Table};
use crate::SeededRng;
use bevy::asset::Assets;
use bevy::image::Image;
use bevy::input::ButtonInput;
use bevy::prelude::{Commands, Component, Entity, KeyCode, Query, Res, ResMut, Single, Sprite};
use rand::RngExt;
use rand_chacha::ChaCha8Rng;

pub fn update_automation_view(
    query: Single<(&Sprite, &SeedAutomation)>,
    mut images: ResMut<Assets<Image>>,
) {


    let (sprite, automation) = query.into_inner();
    let image = automation.world.get_image_data();


    // images.remove(sprite.image.id());
    images.insert(sprite.image.id(), image).unwrap();
}

pub fn update_automation(
    mut query: Single<(&mut SeedAutomation, Entity)>,
    mut seeded_rng: ResMut<SeededRng>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands) {
    

    let  (mut automation, entity) = query.into_inner();

    if keys.just_pressed(KeyCode::Space){
        let random = &mut seeded_rng.0;
        automation.next(random)
    }
    if keys.just_pressed(KeyCode::Enter){
        println!("Switching to PlateAutomation");
        commands.entity(entity).remove::<SeedAutomation>().insert(PlateAutomation{
            world: automation.world.clone()
        });
    }
}

#[derive(Component)]
pub struct SeedAutomation {
    pub(crate) world: Table<bool>,
}

impl SeedAutomation {
    fn next(&mut self, rng: &mut ChaCha8Rng) {
        loop {
            let x = rng.random_range(16..48);
            let y = rng.random_range(16..48);

            if *self.world.get_dim(x, y) {
                continue;
            }
            self.world.set_dim(x, y, true);
            break;
        }
    }
}
