use crate::minmax;

/// A `Canvas` is the surface where we can draw shapes using ASCII characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Canvas {
    pub canvas: Vec<Vec<u8>>,
    pub width: usize,
    pub height: usize,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            canvas: vec![vec![b' '; width]; height],
            width,
            height,
        }
    }

    pub fn draw_rect_outline(&mut self, x: usize, y: usize, width: usize, height: usize) {
        for xx in 0..width {
            self.canvas[y][x + xx] = b'-';
            self.canvas[y + height - 1][x + xx] = b'-';
        }

        for yy in 0..height {
            self.canvas[yy + y][x] = b'|';
            self.canvas[yy + y][x + width - 1] = b'|';
        }

        self.canvas[y][x] = b'+';
        self.canvas[y + height - 1][x] = b'+';
        self.canvas[y][x + width - 1] = b'+';
        self.canvas[y + height - 1][x + width - 1] = b'+';
    }

    pub fn draw_text(&mut self, x: usize, y: usize, text: &[u8]) {
        self.canvas[y][x..x + text.len()].copy_from_slice(text);
    }

    pub fn draw_vertical_line(&mut self, x: usize, (y0, y1): (usize, usize)) {
        let (y0, y1) = minmax(y0, y1);
        for y in (y0..y1).skip(1) {
            self.canvas[y][x] = b'|';
        }
        self.canvas[y0][x] = b'+';
        self.canvas[y1][x] = b'+';
    }

    pub fn draw_horizontal_line(&mut self, y: usize, (x0, x1): (usize, usize)) {
        let (x0, x1) = minmax(x0, x1);
        for x in (x0..x1).skip(1) {
            self.canvas[y][x] = b'-';
        }
        self.canvas[y][x0] = b'+';
        self.canvas[y][x1] = b'+';
    }
}
