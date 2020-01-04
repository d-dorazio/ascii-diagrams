use ascii_diagrams::*;

fn main() {
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

    for l in canvas {
        println!("{}", String::from_utf8(l).unwrap());
    }
}
