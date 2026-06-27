pub struct HexTable {
    len: usize,
    scale: f32,
}

const VERTICAL_OFFSET: f32 = 0.866025;
impl HexTable {
    pub fn new(len: usize, scale: f32) -> Self {
        Self { len, scale }
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

    pub fn around(&self, x: f32, y: f32) -> Vec<(f32, f32)> {
        let around = vec![
            (x + self.scale, y),
            (x - self.scale, y),
            (x + 0.5 * self.scale, y + VERTICAL_OFFSET * self.scale),
            (x - 0.5 * self.scale, y + VERTICAL_OFFSET * self.scale),
            (x + 0.5 * self.scale, y - VERTICAL_OFFSET * self.scale),
            (x - 0.5 * self.scale, y - VERTICAL_OFFSET * self.scale),
        ];

        around
            .iter()
            .filter(|(x, y)| {
                *x >= 0.0
                    && *x < self.scale * self.len as f32
                    && *y >= 0.0
                    && *y < self.scale * self.len as f32
            })
            .map(|(x, y)| (*x, *y))
            .collect()
    }
}
