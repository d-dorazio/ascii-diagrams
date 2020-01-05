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
            &edges,
            RenderOptions {
                hmargin: 5,
                vmargin: 2,
                padding: 1,
            },
        );

        assert_eq!(canvas.join(&b"\n"[..]),
       br#"                                                                                                  
                   +--------------------------------------------+                                 
     +------------+|    +--------------------+     +-------+    |+--------------------------+     
     |            ||    |                    |     |       |    ||                          |     
     |            ||    |                    |     | yolo  |    ||                          |     
     | ciao mondo ++----+ l'ultimo dell'anno +-----+ foo   |    ++ aperitivo della vittoria |     
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
                                                                                                  
                                                                                                  "#.to_vec()
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
            &edges,
            RenderOptions {
                hmargin: 5,
                vmargin: 2,
                padding: 1,
            },
        );

        assert_eq!(canvas.join(&b"\n"[..]),
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
                                                                                                                                                                                            
                                                                                                                                                                                            "#.to_vec()


        );
    }
}
