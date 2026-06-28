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
        let around : Vec<(isize, isize)> = vec![
            (x, y + 1),
            (x + 1, y + 1),
            (x, y - 1),
            (x + 1, y - 1),
            (x - 1, y),
            (x + 1, y),
        ];

        around
            .iter()
            .filter(|(x, y)| {
                *x >= 0
                    && *x <= self.dimensions.0 as isize
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
        self.get(x + y * self.dimensions.0)
    }

    pub fn set(&mut self, index: usize, value: T) {
        self.data[index] = value;
    }

    pub fn set_dim(&mut self, x: usize, y: usize, value: T) {
        self.set(x + y * self.dimensions.0, value);
    }
}
