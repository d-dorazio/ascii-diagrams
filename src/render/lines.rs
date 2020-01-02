use super::canvas::Canvas;
use super::canvas_space::CanvasSpace;
use crate::{minmax, Block};

pub type Polyline = Vec<Line>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    Vertical(usize, (usize, usize)),
    Horizontal(usize, (usize, usize)),
}

/// Try to find the shortest paths that minimize intersections between edges, but that still
/// connect all the boxes as requested.
pub fn find_lines_path(
    _canvas: &Canvas,
    cs: &CanvasSpace,
    boxes: &[Block],
    edges: &[(usize, usize)],
) -> Vec<Polyline> {
    let mut polylines = Vec::with_capacity(edges.len());

    for (b0, b1) in edges {
        let b0 = &boxes[*b0];
        let b1 = &boxes[*b1];
        let (r0, c0) = (b0.row, b0.column);
        let (r1, c1) = (b1.row, b1.column);

        // same row, just draw a horizontal line
        if r0 == r1 {
            let (c0, c1) = minmax(c0, c1);

            polylines.push(vec![Line::Horizontal(
                cs.row_y(r0) + cs.row_height(r0) / 2,
                (cs.column_x(c0) + cs.column_width(c0) - 1, cs.column_x(c1)),
            )]);
            continue;
        }

        // same column, just draw a vertical line
        if c0 == c1 {
            let (r0, r1) = minmax(r0, r1);
            polylines.push(vec![Line::Vertical(
                cs.column_x(c0) + cs.column_width(c0) / 2,
                (cs.row_y(r0) + cs.row_height(r0) - 1, cs.row_y(r1)),
            )]);
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

            let poly = vec![
                Line::Vertical(
                    turn_x,
                    (
                        cs.row_y(r0) + if sr > 0 { cs.row_height(r0) - 1 } else { 0 },
                        turn_y,
                    ),
                ),
                Line::Horizontal(
                    turn_y,
                    (
                        turn_x,
                        cs.column_x(c1) + if sc < 0 { cs.column_width(c1) - 1 } else { 0 },
                    ),
                ),
            ];

            polylines.push(poly);
            continue;
        }

        // horizontal -> vertical
        if cs.free_path((1..=ww).map(|i| (r0, c0 + i * sc)))
            && cs.free_path((0..hh).map(|i| (r0 + i * sr, c1)))
        {
            let turn_y = cs.row_y(r0) + cs.row_height(r0) / 2;
            let turn_x = cs.column_x(c1) + cs.column_width(c1) / 2;

            let poly = vec![
                Line::Horizontal(
                    turn_y,
                    (
                        cs.column_x(c0) + if sc > 0 { cs.column_width(c0) - 1 } else { 0 },
                        turn_x,
                    ),
                ),
                Line::Vertical(
                    turn_x,
                    (
                        turn_y,
                        cs.row_y(r1) + if sr < 0 { cs.row_height(r1) - 1 } else { 0 },
                    ),
                ),
            ];

            polylines.push(poly);
            continue;
        }

        todo!("complex line routing has not been implemented yet");
    }

    polylines
}
