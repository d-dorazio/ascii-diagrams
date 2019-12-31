#[derive(Debug, Clone, PartialEq, Eq)]
struct Block {
    text: Vec<Vec<u8>>,

    // coordinates in a grid world
    column: usize,
    row: usize,

    // rendering data, should probably live elsewhere
    width: usize,
    height: usize,
}

impl Block {
    fn new((row, column): (usize, usize), t: &[u8]) -> Self {
        let mut width = 0;
        let mut text = vec![vec![]];
        for c in t {
            if *c == b'\n' {
                width = width.max(text.last().unwrap().len());
                text.push(vec![]);
                continue;
            }

            if *c == b' ' || c.is_ascii_graphic() {
                text.last_mut().unwrap().push(*c);
            }
        }
        width = width.max(text.last().unwrap().len());

        let height = text.len();

        Self {
            column,
            height,
            row,
            text,
            width,
        }
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderOptions {
    hmargin: usize,
    vmargin: usize,
    padding: usize,
}

fn render(
    boxes: &[Block],
    RenderOptions {
        hmargin,
        vmargin,
        padding,
    }: RenderOptions,
) -> Vec<Vec<u8>> {
    if boxes.is_empty() {
        return vec![];
    }

    let width = 1 + boxes.iter().map(|b| b.column).max().unwrap();
    let height = 1 + boxes.iter().map(|b| b.row).max().unwrap();

    let mut columns_width = vec![0; width];
    let mut rows_height = vec![0; height];

    for b in boxes {
        // +2 to account for borders
        let w = 2 + b.width + padding * 2;
        let h = 2 + b.height + padding * 2;

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

    // subtract a margin to remove trailing empty margin
    let canvas_width = hmargin * width - hmargin + columns_width.iter().sum::<usize>();
    let canvas_height = vmargin * height - vmargin + rows_height.iter().sum::<usize>();

    let mut canvas = Canvas::new(canvas_width, canvas_height);

    for b in boxes {
        let xs = columns_xs[b.column];
        let ys = rows_ys[b.row];

        let w = columns_width[b.column];
        let h = rows_height[b.row];

        canvas.draw_rect_outline(xs, ys, w, h);

        let yoff = (h - b.height) / 2;
        let xoff = (w - b.width) / 2;

        for (ty, t) in b.text.iter().enumerate() {
            canvas.draw_text(xs + xoff, ys + yoff + ty, t);
        }
    }

    // TODO: placeholder just to draw some lines
    for b1 in boxes {
        for b2 in boxes {
            if b1.row == b2.row && b1.column + 1 == b2.column {
                canvas.draw_horizontal_line(
                    rows_ys[b1.row] + rows_height[b1.row] / 2,
                    (
                        columns_xs[b1.column] + columns_width[b1.column] - 1,
                        columns_xs[b2.column],
                    ),
                );
                continue;
            }

            if b1.column == b2.column && b1.row < b2.row {
                canvas.draw_vertical_line(
                    columns_xs[b1.column] + columns_width[b1.column] / 2,
                    (rows_ys[b1.row] + rows_height[b1.row] - 1, rows_ys[b2.row]),
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
