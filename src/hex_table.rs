use crate::table::{IntoImage, Table};
use bevy::asset::RenderAssetUsages;
use bevy::image::Image;
use bevy::prelude::Color;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

#[derive(Clone)]
pub struct HexTable<T: Clone> {
    default: T,
    pub(crate) data: Vec<T>,
    pub(crate) len: usize,
    pub(crate) scale: f32,
    pub(crate) dimensions: (usize, usize),
}

const VERTICAL_OFFSET: f32 = 0.866025;
impl<T: Clone> HexTable<T> {
    pub fn new(len: usize, default: T, scale: f32) -> Self {
        let rows = len - 1;
        let columns = (len as f32 / VERTICAL_OFFSET) as usize;
        Self {
            default: default.clone(),
            len,
            scale,
            data: vec![default; rows * columns],
            dimensions: (rows, columns),
        }
    }

    pub fn calculate(&self) -> Vec<(f32, f32)> {
        let mut vec = Vec::with_capacity(self.len * self.len);
        let mut index = 0;
        loop {
            let x = index % self.len;
            let y = index / self.len;
            let y_point = y as f32 * VERTICAL_OFFSET;

            if y_point as usize >= self.len {
                break;
            }
            let mut x_point = x as f32;
            if y % 2 == 1 {
                x_point += 0.5;
            }
            vec.push((x_point * self.scale, y_point * self.scale));
            index += 1;
        }
        vec
    }

    pub fn around(&self, x: usize, y: usize) -> Vec<&T> {
        let x = x as isize;
        let y = y as isize;
        let around: Vec<(isize, isize)> = vec![
            (x, y - 1),
            (x + 1, y - 1),
            (x - 1, y),
            (x + 1, y),
            (x - 1, y + 1),
            (x, y + 1),
        ];

        around
            .iter()
            .filter(|(x, y)| {
                *x >= 0
                    && *x < self.dimensions.0 as isize
                    && *y >= 0
                    && *y < self.dimensions.1 as isize
            })
            .map(|(x, y)| self.get_dim(*x as usize, *y as usize))
            .collect()
    }

    pub fn get(&self, index: usize) -> &T {
        &self.data[index]
    }

    pub fn get_dim(&self, x: usize, y: usize) -> &T {
        if self.to_index(x, y) == self.data.len() {
            panic!("index out of bounds {x}:{y}");
        }
        self.get(self.to_index(x, y))
    }

    pub fn set(&mut self, index: usize, value: T) {
        self.data[index] = value;
    }

    pub fn set_dim(&mut self, x: usize, y: usize, value: T) {
        self.set(self.to_index(x, y), value);
    }

    pub fn get_offset_of_line(&self, index: usize) -> (f32, f32) {
        self.get_offset_of(index % self.dimensions.0, index / self.dimensions.0)
    }

    pub fn get_offset_of(&self, x: usize, y: usize) -> (f32, f32) {
        let y_f = self.scale * y as f32 * VERTICAL_OFFSET;
        let mut x_f = x as f32 * self.scale;
        if y % 2 == 1 {
            x_f += self.scale / 2.0;
        }
        (x_f, y_f)
    }

    #[inline]
    fn to_index(&self, x: usize, y: usize) -> usize {
        x + y * self.dimensions.0
    }

    #[inline]
    fn to_coordinates(&self, index: usize) -> (usize, usize) {
        (index % self.dimensions.0, index / self.dimensions.0)
    }

    #[inline]
    pub fn get_on_square_table<'a>(&self, x: usize, y: usize, table: &'a Table<T>) -> &'a T  {
        let coordinates = self.get_offset_of(x, y);
        let coordinates = (coordinates.0 as usize, coordinates.1 as usize);
        table.get_dim(coordinates.0, coordinates.1)
    }
}

impl IntoImage for HexTable<bool> {
    fn get_image_data(&self) -> Image {
        let side = (self.len as f32 * self.scale) as u32;
        let mut image = Image::new_fill(
            Extent3d {
                width: side,
                height: side,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );


        self.data
            .iter()
            .zip(0..self.data.len())
            .filter(|(v, _)| **v)
            .for_each(|(_, i)| {
                let coords = self.get_offset_of_line(i);
                let coords = (coords.0 as u32, coords.1 as u32);
                image
                    .set_color_at(coords.0, coords.1, Color::WHITE)
                    .unwrap();
            });

        image
    }
}

impl IntoImage for HexTable<u8> {
    fn get_image_data(&self) -> Image {
        let side = (self.len as f32 * self.scale) as u32;
        let mut image = Image::new_fill(
            Extent3d {
                width: side,
                height: side,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );


        self.data
            .iter()
            .zip(0..self.data.len())
            .filter(|(v, _)| **v != 0 )
            .for_each(|(v, i)| {
                let coords = self.get_offset_of_line(i);
                let coords = (coords.0 as u32, coords.1 as u32);
                image
                    .set_color_at(coords.0, coords.1, Color::srgb_u8(*v, *v, *v))
                    .unwrap();
            });

        image
    }
}
