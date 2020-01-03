mod canvas;
mod canvas_space;
mod lines;

use canvas::Canvas;
use canvas_space::CanvasSpace;
use lines::find_lines_path;

use crate::Block;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderOptions {
    /// horizontal margin placed between columns, also after the last one and before the first one.
    pub hmargin: usize,

    /// vertical margin placed between rows, also after the last one and before the first one.
    pub vmargin: usize,

    /// padding around the text inside the `Block`s.
    pub padding: usize,
}

pub fn render(boxes: &[Block], edges: &[(usize, usize)], config: RenderOptions) -> Vec<Vec<u8>> {
    if boxes.is_empty() {
        return vec![];
    }

    let cs = CanvasSpace::new(boxes, config);
    let mut canvas = Canvas::new(cs.canvas_width(), cs.canvas_height());

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

    for poly in find_lines_path(&canvas, &cs, boxes, edges) {
        for l in poly {
            l.draw(&mut canvas);
        }
    }

    canvas.canvas
}
