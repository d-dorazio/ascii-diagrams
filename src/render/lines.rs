use std::collections::{BinaryHeap, HashSet};

use super::canvas::Canvas;
use super::canvas_space::CanvasSpace;
use crate::Block;

pub type Polyline = Vec<Line>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Line {
    Vertical(usize, (usize, usize)),
    Horizontal(usize, (usize, usize)),
}

/// Try to find the shortest paths that minimize intersections between edges, but that still
/// connect all the boxes as requested.
pub fn find_lines_path(
    canvas: &Canvas,
    cs: &CanvasSpace,
    blocks: &[Block],
    edges: impl IntoIterator<Item = (usize, usize)>,
) -> Vec<Polyline> {
    // convert whatever is on the canvas to walls, lines are not considered walls as other lines
    // can pass on other lines but can never pass inside a block
    let mut canvas = canvas.clone();
    for row in &mut canvas.canvas {
        for c in row {
            if *c != b' ' {
                *c = b'#';
            }
        }
    }

    // if there's enough margin either vertically or horizontally then place fake lines around the
    // borders of the blocks to avoid passing through them if possible
    if cs.render_cfg().hmargin > 2 {
        for b in blocks {
            let x = cs.column_x(b.column);
            let y = cs.row_y(b.row);
            let w = cs.column_width(b.column);
            let h = cs.row_height(b.row);
            for yy in 0..h {
                canvas.canvas[y + yy][x - 1] = b'|';
                canvas.canvas[y + yy][x + w] = b'|';
            }
        }
    }
    if cs.render_cfg().vmargin > 2 {
        for b in blocks {
            let x = cs.column_x(b.column);
            let y = cs.row_y(b.row);
            let w = cs.column_width(b.column);
            let h = cs.row_height(b.row);
            for xx in 0..w {
                canvas.canvas[y - 1][x + xx] = b'-';
                canvas.canvas[y + h][x + xx] = b'-';
            }
        }
    }

    // TODO: score polylines according to turns, intersections, length, etc... and find the
    // configuration with the best score
    initial_polylines(cs, &mut canvas, blocks, edges)
}

fn initial_polylines(
    cs: &CanvasSpace,
    canvas: &mut Canvas,
    blocks: &[Block],
    edges: impl IntoIterator<Item = (usize, usize)>,
) -> Vec<Polyline> {
    let edges = edges.into_iter();
    let mut polylines = Vec::with_capacity(edges.size_hint().0);

    for (b0, b1) in edges {
        let b0 = &blocks[b0];
        let b1 = &blocks[b1];
        let (r0, c0) = (b0.row, b0.column);
        let (r1, c1) = (b1.row, b1.column);

        let mut src: (usize, usize) = (0, 0);
        let mut dst: (usize, usize) = (0, 0);

        if r0 == r1 {
            src.1 = cs.row_y(r0) + cs.row_height(r0) / 2;
            dst.1 = src.1;

            if c0 < c1 {
                src.0 = cs.column_x(c0) + cs.column_width(c0) - 1;
                dst.0 = cs.column_x(c1);
            } else {
                src.0 = cs.column_x(c0);
                dst.0 = cs.column_x(c1) + cs.column_width(c1) - 1;
            }
        } else if c0 == c1 {
            src.0 = cs.column_x(c0) + cs.column_width(c0) / 2;
            dst.0 = src.0;

            if r0 < r1 {
                src.1 = cs.row_y(r0) + cs.row_height(r0) - 1;
                dst.1 = cs.row_y(r1);
            } else {
                src.1 = cs.row_y(r0);
                dst.1 = cs.row_y(r1) + cs.row_height(r1) - 1;
            }
        } else if r0 < r1 {
            dst.1 = cs.row_y(r1) + cs.row_height(r1) / 2;
            src.0 = cs.column_x(c0) + cs.column_width(c0) / 2;

            src.1 = cs.row_y(r0) + cs.row_height(r0) - 1;
            if c0 < c1 {
                dst.0 = cs.column_x(c1);
            } else {
                dst.0 = cs.column_x(c1) + cs.column_width(c1) - 1;
            }
        } else {
            dst.1 = cs.row_y(r1) + cs.row_height(r1) - 1;
            dst.0 = cs.column_x(c1) + cs.column_width(c1) / 2;

            src.1 = cs.row_y(r0) + cs.row_height(r0) / 2;

            if c0 < c1 {
                src.0 = cs.column_x(c0) + cs.column_width(c0) - 1;
            } else {
                src.0 = cs.column_x(c0);
            }
        }

        // try to first find a path without an intersection, but if that cannot be found allow
        // intersections
        let path = shortest_path(cs, canvas, src, dst, false)
            .or_else(|| shortest_path(cs, canvas, src, dst, true));

        match path {
            Some(path) => {
                for l in &path {
                    l.draw(canvas);
                }
                // restore walls
                canvas.canvas[src.1][src.0] = b'#';
                canvas.canvas[dst.1][dst.0] = b'#';

                polylines.push(path);
            }
            None => {
                todo!("no free path even with intersections enabled?");
            }
        }
    }

    polylines
}

fn shortest_path(
    cs: &CanvasSpace,
    canvas: &Canvas,
    src: (usize, usize),
    dst: (usize, usize),
    allow_intersections: bool,
) -> Option<Polyline> {
    let mut seen = HashSet::new();
    let mut queue = BinaryHeap::new();
    queue.push((0, src, vec![]));

    while let Some((d, (x, y), path)) = queue.pop() {
        let d = -d;

        if (x, y) == dst {
            return Some(path);
        }

        if !seen.insert((x, y)) {
            continue;
        }

        let mut push_node = |xx: usize, yy: usize| {
            // always allow to overwrite the point outside src and dst even if there's a line and
            // intersections are not allowed
            let srcd = xx.max(src.0) - xx.min(src.0) + yy.max(src.1) - yy.min(src.1);
            let dstd = xx.max(dst.0) - xx.min(dst.0) + yy.max(dst.1) - yy.min(dst.1);
            if (xx, yy) == src
                || (xx, yy) == dst
                || canvas.canvas[yy][xx] == b' '
                || (canvas.canvas[yy][xx] != b'#' && (srcd == 1 || dstd == 1))
                || (allow_intersections && canvas.canvas[yy][xx] != b'#')
            {
                let mut new_path = path.clone();

                let mut cost = 1;

                // make intersections cost more as we want to minimize them to avoid ambiguities.
                // In particular make them cost more than turns.
                if canvas.canvas[yy][xx] != b' ' {
                    cost += 2;
                }

                // if the new point is on the last line then do not insert a new segment, but
                // extend the last one
                match new_path.last_mut() {
                    Some(Line::Vertical(lx, (ly, lyy))) if *lx == xx => {
                        *ly = y.min(yy).min(*ly);
                        *lyy = y.max(yy).max(*lyy);
                    }
                    Some(Line::Horizontal(ly, (lx, lxx))) if *ly == yy => {
                        *lx = x.min(xx).min(*lx);
                        *lxx = x.max(xx).max(*lxx);
                    }
                    _ => {
                        new_path.push(if yy == y {
                            Line::Horizontal(y, (x.min(xx), xx.max(x)))
                        } else {
                            Line::Vertical(x, (y.min(yy), yy.max(y)))
                        });

                        // make turns cost more as ideally we want to minimize them in order to
                        // have easy to follow graphs
                        cost += 1;
                    }
                }

                queue.push((-(d + cost), (xx, yy), new_path));
            }
        };

        if x > 0 {
            push_node(x - 1, y);
        }
        if x + 1 < cs.canvas_width() {
            push_node(x + 1, y);
        }
        if y + 1 < cs.canvas_height() {
            push_node(x, y + 1);
        }
        if y > 0 {
            push_node(x, y - 1);
        }
    }

    None
}

impl Line {
    pub fn draw(&self, canvas: &mut Canvas) {
        match *self {
            Line::Horizontal(y, xs) => canvas.draw_horizontal_line(y, xs),
            Line::Vertical(x, ys) => canvas.draw_vertical_line(x, ys),
        }
    }
}
