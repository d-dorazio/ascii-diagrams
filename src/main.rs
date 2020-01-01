#[derive(Debug, Clone, PartialEq, Eq)]
struct Block {
    text: Vec<Vec<u8>>,

    // these coordinates are in a logical space where each block occupies the exact same amount of
    // space both horizontally and vertically.
    column: usize,
    row: usize,

    // these are the dimensions in canvas space of the text contained in the block.
    text_width: usize,
    text_height: usize,
}

impl Block {
    fn new((row, column): (usize, usize), t: &[u8]) -> Self {
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
        for y in (y0..y1).skip(1) {
            self.canvas[y][x] = b'|';
        }
        self.canvas[y0][x] = b'+';
        self.canvas[y1][x] = b'+';
    }

    fn draw_horizontal_line(&mut self, y: usize, (x0, x1): (usize, usize)) {
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
    columns_xs: Vec<usize>,
    columns_width: Vec<usize>,

    rows_ys: Vec<usize>,
    rows_height: Vec<usize>,

    canvas_width: usize,
    canvas_height: usize,
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
        // +1 is to go from inclusive coordinates to exclusive
        let width = 1 + boxes.iter().map(|b| b.column).max().unwrap();
        let height = 1 + boxes.iter().map(|b| b.row).max().unwrap();

        let mut columns_width = vec![0; width];
        let mut rows_height = vec![0; height];

        for b in boxes {
            // +2 to account for block borders
            let w = 2 + b.text_width + padding * 2;
            let h = 2 + b.text_height + padding * 2;

            columns_width[b.column] = columns_width[b.column].max(w);
            rows_height[b.row] = rows_height[b.row].max(h);
        }

        let mut columns_xs = vec![0; width];
        for x in 1..width {
            columns_xs[x] = columns_xs[x - 1] + columns_width[x - 1] + hmargin;
        }

        let mut rows_ys = vec![0; height];
        for y in 1..height {
            rows_ys[y] = rows_ys[y - 1] + rows_height[y - 1] + vmargin;
        }

        // subtract a margin to remove the final trailing empty margin
        let canvas_width =
            hmargin * width - hmargin + columns_xs[width - 1] + columns_width[width - 1];
        let canvas_height =
            vmargin * height - vmargin + rows_ys[height - 1] + rows_height[height - 1];

        Self {
            columns_width,
            rows_height,
            columns_xs,
            rows_ys,
            canvas_width,
            canvas_height,
        }
    }

    fn column_x(&self, column: usize) -> usize {
        self.columns_xs[column]
    }
    fn column_width(&self, column: usize) -> usize {
        self.columns_width[column]
    }

    fn row_y(&self, row: usize) -> usize {
        self.rows_ys[row]
    }
    fn row_height(&self, row: usize) -> usize {
        self.rows_height[row]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderOptions {
    hmargin: usize,
    vmargin: usize,
    padding: usize,
}

fn render(boxes: &[Block], config: RenderOptions) -> Vec<Vec<u8>> {
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

    // TODO: placeholder just to draw some lines
    for b1 in boxes {
        for b2 in boxes {
            if b1.row == b2.row && b1.column + 1 == b2.column {
                canvas.draw_horizontal_line(
                    cs.row_y(b1.row) + cs.row_height(b1.row) / 2,
                    (
                        cs.column_x(b1.column) + cs.column_width(b1.column) - 1,
                        cs.column_x(b2.column),
                    ),
                );
                continue;
            }

            if b1.column == b2.column && b1.row < b2.row {
                canvas.draw_vertical_line(
                    cs.column_x(b1.column) + cs.column_width(b1.column) / 2,
                    (
                        cs.row_y(b1.row) + cs.row_height(b1.row) - 1,
                        cs.row_y(b2.row),
                    ),
                );
                continue;
            }
        }
    }

    canvas.canvas
}

fn main() {
    let boxes = [
        Block::new((0, 0), b"ciao mondo"),
        Block::new((1, 0), b"yolo"),
        Block::new((0, 1), b"l'ultimo dell'anno"),
        Block::new((2, 2), b"cacca"),
        Block::new((0, 2), b"yolo\nfoo\nbar"),
    ];

    let canvas = render(
        &boxes,
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
