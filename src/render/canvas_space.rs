use std::collections::HashSet;
use std::convert::TryFrom;

use crate::render::RenderOptions;
use crate::{Block, LogicalCoord, LogicalPoint};

/// `CanvasSpace` is the definition of the cannvas dimensions (columns width and rows height)
/// required to render a set of `Block`s.
///
/// Each `Block` logically occupies a single point, but in `CanvasSpace` it is expanded to the
/// actual dimensions required to be drawn.
///
/// In spirit it is similar to a 3D camera that goes from 3D space to 2D.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanvasSpace {
    min_column: LogicalCoord,
    min_row: LogicalCoord,

    columns_xs: Vec<usize>,
    columns_width: Vec<usize>,

    rows_ys: Vec<usize>,
    rows_height: Vec<usize>,

    canvas_width: usize,
    canvas_height: usize,

    blocks_position: HashSet<LogicalPoint>,
}

impl CanvasSpace {
    pub fn new(
        boxes: &[Block],
        RenderOptions {
            hmargin,
            vmargin,
            padding,
        }: RenderOptions,
    ) -> Self {
        let mut min_column = LogicalCoord::max_value();
        let mut min_row = LogicalCoord::max_value();
        let mut max_column = LogicalCoord::min_value();
        let mut max_row = LogicalCoord::min_value();
        for b in boxes {
            min_column = min_column.min(b.column);
            min_row = min_row.min(b.row);
            max_column = max_column.max(b.column);
            max_row = max_row.max(b.row);
        }

        // +1 is to go from inclusive coordinates to exclusive
        let width = 1 + usize::try_from(max_column - min_column).unwrap();
        let height = 1 + usize::try_from(max_row - min_row).unwrap();

        let mut cs = Self {
            min_column,
            min_row,

            columns_xs: vec![0; width],
            columns_width: vec![0; width],

            rows_ys: vec![0; height],
            rows_height: vec![0; height],

            canvas_width: 0,
            canvas_height: 0,

            blocks_position: HashSet::with_capacity(boxes.len()),
        };

        for b in boxes {
            let c = usize::try_from(b.column - min_column).unwrap();
            let r = usize::try_from(b.row - min_row).unwrap();

            // +2 to account for block borders
            let w = 2 + b.text_width + padding * 2;
            let h = 2 + b.text_height + padding * 2;

            cs.columns_width[c] = cs.columns_width[c].max(w);
            cs.rows_height[r] = cs.rows_height[r].max(h);

            cs.blocks_position.insert((b.row, b.column));
        }

        for x in 1..width {
            cs.columns_xs[x] = cs.columns_xs[x - 1] + cs.columns_width[x - 1] + hmargin;
        }
        for y in 1..height {
            cs.rows_ys[y] = cs.rows_ys[y - 1] + cs.rows_height[y - 1] + vmargin;
        }

        // subtract a margin to remove the final trailing empty margin
        cs.canvas_width =
            hmargin * width - hmargin + cs.columns_xs[width - 1] + cs.columns_width[width - 1];
        cs.canvas_height =
            vmargin * height - vmargin + cs.rows_ys[height - 1] + cs.rows_height[height - 1];

        cs
    }

    pub fn canvas_width(&self) -> usize {
        self.canvas_width
    }
    pub fn canvas_height(&self) -> usize {
        self.canvas_height
    }

    pub fn column_x(&self, column: LogicalCoord) -> usize {
        self.columns_xs[usize::try_from(column - self.min_column).unwrap()]
    }
    pub fn column_width(&self, column: LogicalCoord) -> usize {
        self.columns_width[usize::try_from(column - self.min_column).unwrap()]
    }

    pub fn row_y(&self, row: LogicalCoord) -> usize {
        self.rows_ys[usize::try_from(row - self.min_row).unwrap()]
    }
    pub fn row_height(&self, row: LogicalCoord) -> usize {
        self.rows_height[usize::try_from(row - self.min_row).unwrap()]
    }

    pub fn free_path(&self, path: impl IntoIterator<Item = LogicalPoint>) -> bool {
        path.into_iter().all(|p| !self.blocks_position.contains(&p))
    }
}
