use crate::SeededRng;
use crate::basic::{IntoImage, Table};
use bevy::asset::{Assets, RenderAssetUsages};
use bevy::image::{BevyDefault, Image};
use bevy::input::ButtonInput;
use bevy::prelude::{Commands, Component, Entity, KeyCode, Query, Res, ResMut, Sprite};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use crate::plate_automation::PlateAutomation;

pub fn update_automation_view(
    query: Query<(&Sprite, &SeedAutomation)>,
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
    mut query: Query<(&mut SeedAutomation, Entity)>,
    mut seeded_rng: ResMut<SeededRng>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands) {

    if query.is_empty() {
        return;
    }

    let  (mut automation, entity) = query.iter_mut().next().unwrap();

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
        let len = (self.world.side * self.world.side) as f32;
        loop {
            let index = (rng.random::<f32>() * len) as usize;
            if *self.world.get(index) {
                continue;
            }
            self.world.set(index, true);
            break;
        }
    }
}
