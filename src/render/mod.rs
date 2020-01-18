#[macro_use]
mod canvas;
mod canvas_space;
mod lines;

use canvas::Canvas;
use canvas_space::CanvasSpace;
use lines::find_edges;

use crate::Block;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderOptions {
    /// horizontal margin placed between columns, also after the last one and before the first one.
    pub hmargin: usize,

    /// vertical margin placed between rows, also after the last one and before the first one.
    pub vmargin: usize,

    /// padding around the text inside the `Block`s.
    pub padding: usize,

    /// seed to use to initialize the rng used for rendering heuristics.
    pub seed: Option<u64>,

    /// maximum number of tweaks to find the best arrangements of lines.
    pub max_tweaks: usize,
}

pub fn render(
    boxes: &[Block],
    edges: impl IntoIterator<Item = (usize, usize)>,
    config: RenderOptions,
) -> Vec<Vec<u8>> {
    if boxes.is_empty() {
        return vec![];
    }

    let cs = CanvasSpace::new(boxes, &config);
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

    for poly in find_edges(&canvas, &cs, boxes, edges, &config) {
        for l in poly {
            l.draw(&mut canvas);
        }
    }

    canvas.canvas
}

#[macro_export]
macro_rules! assert_diagram_eq {
    ($ canvas : expr, $ expected : expr) => {{
        let c = $canvas.join(&b"\n"[..]);
        let e = $expected.to_vec();

        if c != e {
            let pretty_canvas = String::from_utf8(c).unwrap();
            let pretty_expected = String::from_utf8(e).unwrap();

            panic!(
                r#"assertion failed `(left == right)`
canvas:
{}

expected canvas:
{}"#,
                pretty_canvas, pretty_expected
            );
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_diagram() {
        let boxes = [
            Block::new((-1, -1), b"ciao mondo"),
            Block::new((0, 0), b"center"),
            Block::new((1, -1), b"yolo"),
            Block::new((-1, 0), b"l'ultimo dell'anno"),
            Block::new((1, 1), b"cacca"),
            Block::new((-1, 1), b"yolo\nfoo\nbar"),
            Block::new((-1, 2), b"aperitivo della vittoria"),
        ];
        let edges = [
            (0, 2),
            (0, 3),
            (1, 3),
            (3, 5),
            (4, 5),
            // (1, 4),
            (0, 6),
            (4, 1),
        ];

        let canvas = render(
            &boxes,
            edges.iter().copied(),
            RenderOptions {
                hmargin: 5,
                vmargin: 2,
                padding: 1,
                seed: Some(0),
                max_tweaks: 0,
            },
        );

        assert_diagram_eq!(canvas,
       br#"                                                                                                  
            +------------------------------------------------------------------+                  
     +------+-----+     +--------------------+     +-------+     +-------------+------------+     
     |            |     |                    |     |       |     |                          |     
     |            |     |                    |     | yolo  |     |                          |     
     | ciao mondo +-----+ l'ultimo dell'anno +-----+ foo   |     | aperitivo della vittoria |     
     |            |     |                    |     | bar   |     |                          |     
     |            |     |                    |     |       |     |                          |     
     +------+-----+     +----------+---------+     +---+---+     +--------------------------+     
            |                      |                   |                                          
            |                      |                   |                                          
            |           +----------+---------+         |                                          
            |           |                    |         |                                          
            |           |       center       |         |                                          
            |           |                    |         |                                          
            |           +----------+---------+         |                                          
            |                      |                   |                                          
            |                      |                   |                                          
     +------+-----+                |               +---+---+                                      
     |            |                |               |       |                                      
     |    yolo    |                +---------------+ cacca |                                      
     |            |                                |       |                                      
     +------------+                                +-------+                                      
                                                                                                  
                                                                                                  "#
        );
    }

    #[test]
    fn test_aoc2019_day25_diagram() {
        let boxes = [
            Block::new((0, 0), b"hull breach"),
            Block::new((-1, 0), b"hot chocolate fountain"),
            Block::new((-1, -1), b"arcade"),
            Block::new((-2, -1), b"warp drive maintenance room"),
            Block::new((-1, 1), b"sick bay"),
            Block::new((-1, 2), b"gift wrapping center"),
            Block::new((-1, 3), b"navigation"),
            Block::new((0, 1), b"observatory"),
            Block::new((0, 2), b"storage room"),
            Block::new((1, 0), b"hallway"),
            Block::new((1, -1), b"holodeck"),
            Block::new((1, -2), b"stables"),
            Block::new((2, 0), b"passages"),
            Block::new((1, 1), b"science lab"),
            Block::new((2, 1), b"kitchen"),
            Block::new((1, 2), b"corridor"),
            Block::new((2, 2), b"engineering room"),
            Block::new((2, 4), b"crew quarters"),
            Block::new((1, 4), b"security checkpoint"),
            Block::new((1, 3), b"activation pad"),
        ];
        let edges = [
            (0, 1),
            (1, 2),
            (2, 3),
            (1, 4),
            (4, 5),
            (5, 6),
            (0, 7),
            (7, 8),
            (0, 9),
            (9, 10),
            (10, 11),
            (7, 13),
            (13, 14),
            (9, 12),
            (13, 15),
            (15, 16),
            (16, 17),
            (17, 18),
            (18, 19),
        ];

        let canvas = render(
            &boxes,
            edges.iter().copied(),
            RenderOptions {
                hmargin: 5,
                vmargin: 2,
                padding: 1,
                seed: Some(0),
                max_tweaks: 0,
            },
        );

        assert_diagram_eq!(canvas,
        br#"                                                                                                                                                                                            
                                                                                                                                                                                            
                     +-----------------------------+                                                                                                                                        
                     |                             |                                                                                                                                        
                     | warp drive maintenance room |                                                                                                                                        
                     |                             |                                                                                                                                        
                     +--------------+--------------+                                                                                                                                        
                                    |                                                                                                                                                       
                                    |                                                                                                                                                       
                     +--------------+--------------+     +------------------------+     +-------------+     +----------------------+     +----------------+                                 
                     |                             |     |                        |     |             |     |                      |     |                |                                 
                     |           arcade            +-----+ hot chocolate fountain +-----+  sick bay   +-----+ gift wrapping center +-----+   navigation   |                                 
                     |                             |     |                        |     |             |     |                      |     |                |                                 
                     +-----------------------------+     +------------+-----------+     +-------------+     +----------------------+     +----------------+                                 
                                                                      |                                                                                                                     
                                                                      |                                                                                                                     
                                                         +------------+-----------+     +-------------+     +----------------------+                                                        
                                                         |                        |     |             |     |                      |                                                        
                                                         |      hull breach       +-----+ observatory +-----+     storage room     |                                                        
                                                         |                        |     |             |     |                      |                                                        
                                                         +------------+-----------+     +------+------+     +----------------------+                                                        
                                                                      |                        |                                                                                            
                                                                      |                        |                                                                                            
     +---------+     +-----------------------------+     +------------+-----------+     +------+------+     +----------------------+     +----------------+     +---------------------+     
     |         |     |                             |     |                        |     |             |     |                      |     |                |     |                     |     
     | stables +-----+          holodeck           +-----+        hallway         |     | science lab +-----+       corridor       |     | activation pad +-----+ security checkpoint |     
     |         |     |                             |     |                        |     |             |     |                      |     |                |     |                     |     
     +---------+     +-----------------------------+     +------------+-----------+     +------+------+     +-----------+----------+     +----------------+     +----------+----------+     
                                                                      |                        |                        |                                                  |                
                                                                      |                        |                        |                                                  |                
                                                         +------------+-----------+     +------+------+     +-----------+----------+                            +----------+----------+     
                                                         |                        |     |             |     |                      |                            |                     |     
                                                         |        passages        |     |   kitchen   |     |   engineering room   +----------------------------+    crew quarters    |     
                                                         |                        |     |             |     |                      |                            |                     |     
                                                         +------------------------+     +-------------+     +----------------------+                            +---------------------+     
                                                                                                                                                                                            
                                                                                                                                                                                            "#


        );
    }

    #[test]
    fn test_lines_do_not_touch_walls_if_enough_margin() {
        let blocks = [
            Block::new((0, 0), b"zero"),
            Block::new((0, 1), b"one"),
            Block::new((0, 2), b"two"),
            Block::new((1, 2), b"four"),
            Block::new((1, 0), b"0000"),
            Block::new((2, 0), b"oooo"),
        ];

        let edges = [(1, 3), (2, 0), (1, 4), (0, 5)];

        let canvas = render(
            &blocks,
            edges.iter().copied(),
            RenderOptions {
                hmargin: 5,
                vmargin: 3,
                padding: 1,
                seed: Some(0),
                max_tweaks: 0,
            },
        );

        assert_diagram_eq!(
            canvas,
            br#"                                           
         +------------------------+        
         |                        |        
     +---+--+     +-----+     +---+--+     
     |      |     |     |     |      |     
   +-+ zero |   +-+ one |     | two  |     
   | |      |   | |     |     |      |     
   | +------+   | +--+--+     +------+     
   |            |    |                     
   |     +------+    |                     
   |     |           |                     
   | +---+--+        |        +------+     
   | |      |        |        |      |     
   | | 0000 |        +--------+ four |     
   | |      |                 |      |     
   | +------+                 +------+     
   |                                       
   |                                       
   |                                       
   | +------+                              
   | |      |                              
   +-+ oooo |                              
     |      |                              
     +------+                              
                                           
                                           
                                           "#
        );
    }

    #[test]
    fn test_diagram_avoid_intersections_with_straight_line() {
        let blocks = [
            Block::new((0, 0), b"left"),
            Block::new((0, 1), b"center"),
            Block::new((0, 2), b"right"),
            Block::new((1, 1), b"bottom"),
        ];

        let edges = [(0, 2), (1, 3)];

        let canvas = render(
            &blocks,
            edges.iter().copied(),
            RenderOptions {
                hmargin: 5,
                vmargin: 3,
                padding: 1,
                seed: Some(0),
                max_tweaks: 0,
            },
        );

        assert_diagram_eq!(
            canvas,
            br#"                                               
         +---------------------------+         
         |                           |         
     +---+--+     +--------+     +---+---+     
     |      |     |        |     |       |     
     | left |     | center |     | right |     
     |      |     |        |     |       |     
     +------+     +----+---+     +-------+     
                       |                       
                       |                       
                       |                       
                  +----+---+                   
                  |        |                   
                  | bottom |                   
                  |        |                   
                  +--------+                   
                                               
                                               
                                               "#
        );
    }
}
