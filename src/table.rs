use std::ops::{Index, IndexMut};
use bevy::asset::RenderAssetUsages;
use bevy::image::BevyDefault;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub struct Table<T: Clone> {
    default: T,
    pub(crate) data: Vec<T>,
    pub(crate) side: usize,
}

impl<T: Clone> Clone for Table<T> {
    fn clone(&self) -> Self {
        Table {
            default: self.default.clone(),
            data: self.data.clone(),
            side: self.side,
        }
    }
}

impl<T: Clone> Table<T> {
    pub fn new(fill: T, side: usize) -> Self {
        Table {
            default: fill.clone(),
            data: vec![fill; side * side],
            side,
        }
    }

    pub fn get(&self, index: usize) -> &T {
        &self.data[index]
    }

    pub fn get_dim(&self, x: usize, y: usize) -> &T {
        self.get(x + y * self.side)
    }

    pub fn set(&mut self, index: usize, value: T) {
        self.data[index] = value;
    }

    pub fn set_dim(&mut self, x: usize, y: usize, value: T) {
        self.set(x + y * self.side, value);
    }

    pub fn grow(&mut self) {
        let new_side = self.side * 2;
        let mut temp_table = Table::new(self.default.clone(), new_side);
        for x in 0..self.side {
            for y in 0..self.side {
                let new_x = x * 2;
                let new_y = y * 2;
                let val = self.get_dim(x, y);
                temp_table.set_dim(new_x, new_y, val.clone());
                temp_table.set_dim(new_x + 1, new_y, val.clone());
                temp_table.set_dim(new_x, new_y + 1, val.clone());
                temp_table.set_dim(new_x + 1, new_y + 1, val.clone());
            }
        }
        self.data = temp_table.data;
        self.side = new_side;
    }

    pub fn iter(&self) -> impl Iterator<Item=&T> {
        self.data.iter()
    }

    pub fn convert_copy<X: Clone>(&self, table: &mut Table<X>, f: impl Fn(T) -> X) {
        for i in 0..self.data.len() {
            table.data[i] = f(self[i].clone())
        }
    }

    pub fn around_line(&self, index: usize) -> Vec<&T> {
        self.around(index % self.side, index / self.side)
    }


    pub fn around(&self, x: usize, y: usize) -> Vec<&T> {
        let coordinates = vec![
            (x as isize, y as isize - 1),
            (x as isize, y as isize + 1),
            (x as isize + 1, y as isize),
            (x as isize - 1, y as isize),
        ];
        coordinates.iter().filter(|(x, y)| {
            *x >= 0 && *x < self.side as isize && *y >= 0 && *y < self.side as isize
        }).map(|(x, y)| self.get_dim(*x as usize, *y as usize)).collect::<Vec<_>>()
    }
}

impl<T: Clone> Into<Vec<T>> for Table<T> {
    fn into(self) -> Vec<T> {
        self.data
    }
}

pub trait IntoImage {
    fn get_image_data(&self) -> Image;
}

impl IntoImage for Table<bool> {
    fn get_image_data(&self) -> Image {
        let mut data: Vec<u8> = vec![0; self.side * self.side * 4];
        self.data
            .iter()
            .zip((0..self.data.len()).collect::<Vec<_>>())
            .for_each(|(color, idx)| {
                let idx = idx * 4;
                let value = if *color { 255 } else { 0 };

                data[idx] = value;
                data[idx + 1] = value;
                data[idx + 2] = value;
                data[idx + 3] = value;
            });
        Image::new(
            Extent3d {
                width: self.side as u32,
                height: self.side as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        )
    }
}
impl IntoImage for Table<Color> {
    fn get_image_data(&self) -> Image {
        let mut data: Vec<u8> = vec![0; self.side * self.side * 4];
        self.data
            .iter()
            .zip((0..self.data.len()).collect::<Vec<_>>())
            .for_each(|(color, idx)| {
                let idx = idx * 4;
                let color = color.to_linear();
                data[idx] = (color.red * 256.0) as u8;
                data[idx + 1] = (color.green * 256.0) as u8;
                data[idx + 2] = (color.blue * 256.0) as u8;
                data[idx + 3] = (color.alpha * 256.0) as u8;
            });
        Image::new(
            Extent3d {
                width: self.side as u32,
                height: self.side as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        )
    }
}

impl IntoImage for Table<u8> {
    fn get_image_data(&self) -> Image {
        let mut data: Vec<u8> = vec![0; self.side * self.side * 4];
        self.data
            .iter()
            .zip((0..self.data.len()).collect::<Vec<_>>())
            .for_each(|(color, idx)| {
                let idx = idx * 4;

                data[idx] = *color;
                data[idx + 1] = *color;
                data[idx + 2] = *color;
                data[idx + 3] = if *color != 0 { 255 } else { 0 };
            });

        Image::new(
            Extent3d {
                width: self.side as u32,
                height: self.side as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        )
    }
}

impl<T: Clone> Index<usize> for Table<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl<T: Clone> IndexMut<usize> for Table<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}
