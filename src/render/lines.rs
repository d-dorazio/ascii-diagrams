use std::collections::{BinaryHeap, HashSet};

use crate::render::canvas::{Canvas, CanvasPoint};
use crate::render::canvas_space::CanvasSpace;
use crate::{Block, LogicalPoint};

pub type Polyline = Vec<Line>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Line {
    Vertical(usize, (usize, usize)),
    Horizontal(usize, (usize, usize)),
}

/// Try to find the shortest paths that minimize intersections between edges, but that still
/// connect all the boxes as requested.
pub fn find_edges(
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

    let mut edges = edges.into_iter().collect::<Vec<_>>();

    // sort edges by length in order to place the shortest edges first as we have less chance to
    // get them wrong (especially if they're between adjacent blocks)
    edges.sort_by_key(|(b0, b1)| {
        let b0 = &blocks[*b0];
        let b1 = &blocks[*b1];
        (b0.column - b1.column).abs() + (b0.row - b1.row).abs()
    });

    // TODO: find the best arragement of lines by (pseudo-)randomly changing the lines and scoring
    // the final solution looking at different heuristics like maximum line length, number of
    // intersections, minimum distance from a line and a block, etc...

    connect_edges(cs, &mut canvas, blocks, &edges)
}

fn connect_edges(
    cs: &CanvasSpace,
    canvas: &mut Canvas,
    blocks: &[Block],
    edges: &[(usize, usize)],
) -> Vec<Polyline> {
    let mut polylines = Vec::with_capacity(edges.len());

    for (b0, b1) in edges {
        let b0 = &blocks[*b0];
        let b1 = &blocks[*b1];

        let s = (b0.row, b0.column);
        let d = (b1.row, b1.column);

        // try to connect the edge from src to dst and viceversa because the connection points
        // might be different in case the edge is not straight.
        let (p0, p1) = closest_block_points(cs, s, d);
        let (q0, q1) = closest_block_points(cs, d, s);
        let has_alternative = p0 != q1 || p1 != q0;

        // always prefer paths that do not create intersections because the final diagram is
        // easier to follow given that we need to just follow the lines.
        let path = [false, true]
            .iter()
            .filter_map(|&allow_intersections| {
                let path = shortest_path(cs, canvas, p0, p1, allow_intersections);
                if !has_alternative {
                    return path;
                }

                let inv = shortest_path(cs, canvas, q0, q1, allow_intersections);

                match (path, inv) {
                    (Some((p, sp)), Some((q, sq))) => {
                        Some(if sp <= sq { (p, sp) } else { (q, sq) })
                    }
                    (Some(p), _) => Some(p),
                    (_, Some(q)) => Some(q),
                    (None, None) => None,
                }
            })
            .next();

        match path {
            Some((path, _score)) => {
                for l in &path {
                    l.draw(canvas);
                }
                polylines.push(path);
            }
            None => {
                unreachable!("no free path even with intersections enabled?");
            }
        }
    }

    polylines
}

fn closest_block_points(
    cs: &CanvasSpace,
    (r0, c0): LogicalPoint,
    (r1, c1): LogicalPoint,
) -> (CanvasPoint, CanvasPoint) {
    if r0 == r1 {
        // +--+   +--+
        // |s0+---+d0|
        // +--+   +--+

        let mut src = (0, cs.row_y(r0) + cs.row_height(r0) / 2);
        let mut dst = (0, src.1);

        if c0 < c1 {
            src.0 = cs.column_x(c0) + cs.column_width(c0) - 1;
            dst.0 = cs.column_x(c1);
        } else {
            src.0 = cs.column_x(c0);
            dst.0 = cs.column_x(c1) + cs.column_width(c1) - 1;
        }

        return (src, dst);
    }

    if c0 == c1 {
        // +--+
        // |d0|
        // +-++
        //   |
        // +-++
        // |s0|
        // +--+

        let mut src = (cs.column_x(c0) + cs.column_width(c0) / 2, 0);
        let mut dst = (src.0, 0);

        if r0 < r1 {
            src.1 = cs.row_y(r0) + cs.row_height(r0) - 1;
            dst.1 = cs.row_y(r1);
        } else {
            src.1 = cs.row_y(r0);
            dst.1 = cs.row_y(r1) + cs.row_height(r1) - 1;
        }

        return (src, dst);
    }

    //
    // +--+                    +--+             +--+      +--+
    // |s0|                    |s1|        +----+d2|      |d3+-----+
    // +-++                    +-++        |    +--+      +--+     |
    //   |                       |         |                       |
    //   |    +--+      +--+     |       +-++                    +-++
    //   +----+d0|      |d1+-----+       |s2|                    |s3|
    //        +--+      +--+             +--+                    +--+
    //
    //
    // Note that these patterns are not reversible that is
    // `block_points(src, dst) != block_points(dst, src)` but that's ok as it provides a nice hook
    // to force the layout of the line.
    //

    let src = (
        cs.column_x(c0) + cs.column_width(c0) / 2,
        if r0 < r1 {
            cs.row_y(r0) + cs.row_height(r0) - 1
        } else {
            cs.row_y(r0)
        },
    );

    let dst = (
        if c0 < c1 {
            cs.column_x(c1)
        } else {
            cs.column_x(c1) + cs.column_width(c1) - 1
        },
        cs.row_y(r1) + cs.row_height(r1) / 2,
    );

    (src, dst)
}

fn shortest_path(
    cs: &CanvasSpace,
    canvas: &Canvas,
    src: (usize, usize),
    dst: (usize, usize),
    allow_intersections: bool,
) -> Option<(Polyline, i64)> {
    let mut seen = HashSet::new();
    let mut queue = BinaryHeap::new();
    queue.push((0, src, vec![]));

    while let Some((score, (x, y), path)) = queue.pop() {
        let score = -score;

        if (x, y) == dst {
            return Some((path, score));
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
                || canvas.at((xx, yy)) == b' '
                || (canvas.at((xx, yy)) != b'#' && (srcd <= 1 || dstd <= 1))
                || (allow_intersections && canvas.at((xx, yy)) != b'#')
            {
                let mut new_path = path.clone();

                let mut cost = 1;

                // make intersections cost more as we want to minimize them to avoid ambiguities.
                // In particular make them cost more than turns.
                if canvas.at((xx, yy)) != b' ' {
                    cost += 10;
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

                queue.push((-(score + cost), (xx, yy), new_path));
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::render::canvas_space::CanvasSpace;
    use crate::render::RenderOptions;

    #[test]
    fn test_closest_block_points() {
        //                //
        //  +---+  +---+  //
        //  |000|  |111|  //
        //  +---+  +---+  //
        //                //
        //  +---+         //
        //  |222|         //
        //  +---+         //
        //                //
        //         +---+  //
        //         |333|  //
        //         +---+  //
        //                //

        let blocks = [
            Block::new((0, 0), b"000"),
            Block::new((0, 1), b"111"),
            Block::new((1, 0), b"222"),
            Block::new((2, 1), b"333"),
        ];
        let cs = CanvasSpace::new(
            &blocks,
            RenderOptions {
                hmargin: 2,
                vmargin: 1,
                padding: 0,
            },
        );

        // 000 <-> 111
        assert_eq!(closest_block_points(&cs, (0, 0), (0, 1)), ((6, 2), (9, 2)));
        assert_eq!(closest_block_points(&cs, (0, 1), (0, 0)), ((9, 2), (6, 2)));

        // 111 <-> 333
        assert_eq!(
            closest_block_points(&cs, (2, 1), (0, 1)),
            ((11, 9), (11, 3))
        );
        assert_eq!(
            closest_block_points(&cs, (0, 1), (2, 1)),
            ((11, 3), (11, 9))
        );

        // 222 -> 111
        assert_eq!(closest_block_points(&cs, (1, 0), (0, 1)), ((4, 5), (9, 2)));

        // 111 -> 222
        assert_eq!(closest_block_points(&cs, (0, 1), (1, 0)), ((11, 3), (6, 6)));

        // 222 -> 333
        assert_eq!(closest_block_points(&cs, (1, 0), (2, 1)), ((4, 7), (9, 10)));

        // 333 -> 222
        assert_eq!(closest_block_points(&cs, (2, 1), (1, 0)), ((11, 9), (6, 6)));
    }
}
