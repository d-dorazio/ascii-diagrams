/// (row, column)
pub type LogicalPoint = (LogicalCoord, LogicalCoord);
pub type LogicalCoord = i32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    text: Vec<Vec<u8>>,

    // these coordinates are in a logical space where each block occupies the exact same amount of
    // space both horizontally and vertically.
    column: LogicalCoord,
    row: LogicalCoord,

    // these are the dimensions in canvas space of the text contained in the block.
    text_width: usize,
    text_height: usize,
}

impl Block {
    pub fn new((row, column): LogicalPoint, t: &[u8]) -> Self {
        let mut text_width = 0;
        let mut text = vec![vec![]];
        for c in t {
            if *c == b'\n' {
                text_width = text_width.max(text.last().unwrap().len());
                text.push(vec![]);
                continue;
            }

            if *c == b' ' || c.is_ascii_graphic() {
                text.last_mut().unwrap().push(*c);
            }
        }
        text_width = text_width.max(text.last().unwrap().len());

        let text_height = text.len();

        Self {
            column,
            row,
            text,
            text_height,
            text_width,
        }
    }
}

mod render;

pub use render::{render, RenderOptions};

pub fn minmax<T: Ord>(a: T, b: T) -> (T, T) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}
