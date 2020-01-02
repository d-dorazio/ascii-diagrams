use std::collections::HashSet;
use std::convert::TryFrom;

/// (row, column)
type LogicalPoint = (LogicalCoord, LogicalCoord);
type LogicalCoord = i32;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Block {
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
    fn new((row, column): LogicalPoint, t: &[u8]) -> Self {
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

/// A `Canvas` is the surface where we can draw shapes using ASCII characters.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Canvas {
    canvas: Vec<Vec<u8>>,
    width: usize,
    height: usize,
}

impl Canvas {
    fn new(width: usize, height: usize) -> Self {
        Self {
            canvas: vec![vec![b' '; width]; height],
            width,
            height,
        }
    }

    fn draw_rect_outline(&mut self, x: usize, y: usize, width: usize, height: usize) {
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

    fn draw_text(&mut self, x: usize, y: usize, text: &[u8]) {
        self.canvas[y][x..x + text.len()].copy_from_slice(text);
    }

    fn draw_vertical_line(&mut self, x: usize, (y0, y1): (usize, usize)) {
        let (y0, y1) = minmax(y0, y1);
        for y in (y0..y1).skip(1) {
            self.canvas[y][x] = b'|';
        }
        self.canvas[y0][x] = b'+';
        self.canvas[y1][x] = b'+';
    }

    fn draw_horizontal_line(&mut self, y: usize, (x0, x1): (usize, usize)) {
        let (x0, x1) = minmax(x0, x1);
        for x in (x0..x1).skip(1) {
            self.canvas[y][x] = b'-';
        }
        self.canvas[y][x0] = b'+';
        self.canvas[y][x1] = b'+';
    }
}

/// `CanvasSpace` is the definition of the cannvas dimensions (columns width and rows height)
/// required to render a set of `Block`s.
///
/// Each `Block` logically occupies a single point, but in `CanvasSpace` it is expanded to the
/// actual dimensions required to be drawn.
///
/// In spirit it is similar to a 3D camera that goes from 3D space to 2D.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CanvasSpace {
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
    fn new(
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

    fn column_x(&self, column: LogicalCoord) -> usize {
        self.columns_xs[usize::try_from(column - self.min_column).unwrap()]
    }
    fn column_width(&self, column: LogicalCoord) -> usize {
        self.columns_width[usize::try_from(column - self.min_column).unwrap()]
    }

    fn row_y(&self, row: LogicalCoord) -> usize {
        self.rows_ys[usize::try_from(row - self.min_row).unwrap()]
    }
    fn row_height(&self, row: LogicalCoord) -> usize {
        self.rows_height[usize::try_from(row - self.min_row).unwrap()]
    }

    fn free_path(&self, path: impl IntoIterator<Item = LogicalPoint>) -> bool {
        path.into_iter().all(|p| !self.blocks_position.contains(&p))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderOptions {
    hmargin: usize,
    vmargin: usize,
    padding: usize,
}

fn render(boxes: &[Block], lines: &[(usize, usize)], config: RenderOptions) -> Vec<Vec<u8>> {
    if boxes.is_empty() {
        return vec![];
    }

    let cs = CanvasSpace::new(boxes, config);
    let mut canvas = Canvas::new(cs.canvas_width, cs.canvas_height);

    for b in boxes {
        let x = cs.column_x(b.column);
        let y = cs.row_y(b.row);
        let w = cs.column_width(b.column);
        let h = cs.row_height(b.row);

        canvas.draw_rect_outline(x, y, w, h);

        // center text horizontally and vertically
        let xoff = (w - b.text_width) / 2;
        let yoff = (h - b.text_height) / 2;

        for (ty, t) in b.text.iter().enumerate() {
            canvas.draw_text(x + xoff, y + yoff + ty, t);
        }
    }

    for (b0, b1) in lines {
        let b0 = &boxes[*b0];
        let b1 = &boxes[*b1];
        let (r0, c0) = (b0.row, b0.column);
        let (r1, c1) = (b1.row, b1.column);

        // same row, just draw a horizontal line
        if r0 == r1 {
            let (c0, c1) = minmax(c0, c1);
            canvas.draw_horizontal_line(
                cs.row_y(r0) + cs.row_height(r0) / 2,
                (cs.column_x(c0) + cs.column_width(c0) - 1, cs.column_x(c1)),
            );
            continue;
        }

        // same column, just draw a vertical line
        if c0 == c1 {
            let (r0, r1) = minmax(r0, r1);
            canvas.draw_vertical_line(
                cs.column_x(c0) + cs.column_width(c0) / 2,
                (cs.row_y(r0) + cs.row_height(r0) - 1, cs.row_y(r1)),
            );
            continue;
        }

        // here things are more complicated because the connection must do a turn at least to get
        // to the target. As of now, just check if it's possible to draw a line with a single turn.
        // Collision detection will come...
        let (ww, hh) = ((c1 - c0).abs(), (r1 - r0).abs());
        let (sc, sr) = ((c1 - c0).signum(), (r1 - r0).signum());

        // vertical -> horizontal
        if cs.free_path((1..=hh).map(|i| (r0 + i * sr, c0)))
            && cs.free_path((0..ww).map(|i| (r1, c0 + i * sc)))
        {
            let turn_y = cs.row_y(r1) + cs.row_height(r1) / 2;
            let turn_x = cs.column_x(c0) + cs.column_width(c0) / 2;

            canvas.draw_vertical_line(
                turn_x,
                (
                    cs.row_y(r0) + if sr > 0 { cs.row_height(r0) - 1 } else { 0 },
                    turn_y,
                ),
            );

            canvas.draw_horizontal_line(
                turn_y,
                (
                    turn_x,
                    cs.column_x(c1) + if sc < 0 { cs.column_width(c1) - 1 } else { 0 },
                ),
            );

            continue;
        }

        // horizontal -> vertical
        if cs.free_path((1..=ww).map(|i| (r0, c0 + i * sc)))
            && cs.free_path((0..hh).map(|i| (r0 + i * sr, c1)))
        {
            let turn_y = cs.row_y(r0) + cs.row_height(r0) / 2;
            let turn_x = cs.column_x(c1) + cs.column_width(c1) / 2;

            canvas.draw_horizontal_line(
                turn_y,
                (
                    cs.column_x(c0) + if sc > 0 { cs.column_width(c0) - 1 } else { 0 },
                    turn_x,
                ),
            );

            canvas.draw_vertical_line(
                turn_x,
                (
                    turn_y,
                    cs.row_y(r1) + if sr < 0 { cs.row_height(r1) - 1 } else { 0 },
                ),
            );

            continue;
        }

        todo!("complex line routing has not been implemented yet");
    }

    canvas.canvas
}

fn main() {
    let boxes = [
        Block::new((-1, -1), b"ciao mondo"),
        Block::new((0, 0), b"center"),
        Block::new((1, -1), b"yolo"),
        Block::new((-1, 0), b"l'ultimo dell'anno"),
        Block::new((1, 1), b"cacca"),
        Block::new((-1, 1), b"yolo\nfoo\nbar"),
    ];
    let lines = [
        (0, 2),
        (0, 3),
        (1, 3),
        (3, 5),
        (4, 5),
        (1, 4),
        // (4, 1),
    ];

    let canvas = render(
        &boxes,
        &lines,
        RenderOptions {
            hmargin: 5,
            vmargin: 2,
            padding: 1,
        },
    );

    for l in canvas {
        println!("{}", String::from_utf8(l).unwrap());
    }
}

pub fn minmax<T: Ord>(a: T, b: T) -> (T, T) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}
