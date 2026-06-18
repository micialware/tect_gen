use bevy::prelude::Color;
use std::ops::{Index, IndexMut};

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

    fn get_dim(&mut self, x: usize, y: usize) -> &T {
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

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }

    pub fn convert_copy<X: Clone>(&self, table: &mut Table<X>, f: impl Fn(T) -> X) {
        for i in 0..self.data.len() {
            table.data[i] = f(self[i].clone())
        }
    }
}

impl<T: Clone> Into<Vec<T>> for Table<T> {
    fn into(self) -> Vec<T> {
        self.data
    }
}

pub trait IntoImage {
    fn get_image_data(&self) -> Vec<u8>;
}

impl IntoImage for Table<bool> {
    fn get_image_data(&self) -> Vec<u8> {
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
                data[idx + 3] = 255;
            });
        data
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
