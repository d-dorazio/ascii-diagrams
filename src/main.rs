use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::PathBuf;

use serde::Deserialize;
use structopt::StructOpt;

use ascii_diagrams::*;

macro_rules! try_or_die {
    ($ expr : expr) => {
        match $expr {
            Ok(o) => o,
            Err(e) => {
                if cfg!(test) {
                    panic!("{}", e);
                } else {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }
    };
}

fn default_hmargin() -> usize {
    5
}
fn default_vmargin() -> usize {
    3
}
fn default_padding() -> usize {
    1
}

#[derive(Debug, StructOpt)]
struct Opts {
    #[structopt(name = "INPUT", parse(from_os_str))]
    diagram: Option<PathBuf>,

    #[structopt(name = "OUTPUT", parse(from_os_str))]
    output: Option<PathBuf>,
}

#[derive(Deserialize)]
struct Spec {
    #[serde(default)]
    name: String,

    #[serde(default)]
    description: String,

    blocks: Vec<SpecBlock>,
    edges: Vec<SpecEdge>,

    #[serde(default = "default_hmargin")]
    horizontal_margin: usize,

    #[serde(default = "default_vmargin")]
    vertical_margin: usize,

    #[serde(default = "default_padding")]
    padding: usize,
}

#[derive(Deserialize)]
struct SpecBlock {
    id: Option<String>,
    text: String,
    position: SpecPosition,
}

#[derive(Deserialize)]
struct SpecEdge {
    from: String,
    to: String,
}

#[derive(Deserialize)]
struct SpecPosition {
    row: LogicalCoord,
    column: LogicalCoord,
}

fn main() {
    let opts = Opts::from_args();

    let mut input_spec = vec![];
    match opts.diagram {
        None => {
            try_or_die!(io::stdin().read_to_end(&mut input_spec));
        }
        Some(p) => {
            let mut f = try_or_die!(File::open(p));
            try_or_die!(f.read_to_end(&mut input_spec));
        }
    }

    let canvas = render_diagram(&input_spec);

    match opts.output {
        Some(output) => {
            let mut f = try_or_die!(File::create(output));
            for l in canvas {
                try_or_die!(f.write_all(&l));
                try_or_die!(writeln!(f, ""));
            }
        }
        None => {
            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            for l in canvas {
                try_or_die!(stdout.write_all(&l));
                try_or_die!(writeln!(stdout, ""));
            }
        }
    }
}

fn render_diagram(input_spec: &[u8]) -> Vec<Vec<u8>> {
    let spec: Spec = try_or_die!(toml::from_slice(&input_spec));

    let mut id_to_block_id = HashMap::with_capacity(spec.blocks.len());
    let mut occupied_positions = HashSet::with_capacity(spec.blocks.len());
    let mut blocks = Vec::with_capacity(spec.blocks.len());

    for b in &spec.blocks {
        let id = b.id.as_ref().unwrap_or_else(|| &b.text);
        if id_to_block_id.insert(id, blocks.len()).is_some() {
            println!(r#"duplicate id found: "{}""#, id);
            continue;
        }

        let pos = (b.position.row, b.position.column);
        if !occupied_positions.insert(pos) {
            println!(
                r#"more than one cell present at row {} and column {}"#,
                b.position.row, b.position.column
            );
            continue;
        }

        blocks.push(Block::new(pos, b.text.as_bytes()));
    }

    let mut edges = BTreeSet::new();
    for e in &spec.edges {
        let from = match id_to_block_id.get(&e.from) {
            Some(i) => *i,
            None => {
                println!(r#"id "{}" not found"#, e.from);
                continue;
            }
        };

        let to = match id_to_block_id.get(&e.to) {
            Some(i) => *i,
            None => {
                println!(r#"id "{}" not found"#, e.to);
                continue;
            }
        };

        if edges.contains(&(from, to)) || edges.contains(&(to, from)) {
            println!(r#"duplicate edges from "{}" to "{}""#, e.from, e.to);
            continue;
        }

        edges.insert((from, to));
    }

    render(
        &blocks,
        edges,
        RenderOptions {
            hmargin: spec.horizontal_margin,
            vmargin: spec.vertical_margin,
            padding: spec.padding,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_toml_diagram() {
        let diagram = br#"
name = "example1"

edges = [ {from = "one", to = "four"}
        , {from = "one", to = "0000"}
        , {from = "two", to = "zero"}
        , {from = "oooo", to = "zero"}
        ]

[[blocks]]
text = "zero"
position = { row = -1, column = -1 }

[[blocks]]
text = "one"
position = { row = -1, column = 0 }

[[blocks]]
text = "two"
position = { row = -1, column = 1 }

[[blocks]]
text = "0000"
position = { row = 0, column = -1 }

[[blocks]]
text = "four"
position = { row = 0, column = 1 }

[[blocks]]
text = "oooo"
position = { row = 1, column = -1 }

"#;

        assert_eq!(
            render_diagram(diagram).join(&b"\n"[..]),
            br#"                                           
              +-----------+                
              |           |                
     +------+ |   +-----+ |   +------+     
     |      | |   |     | |   |      |     
     | zero +-+   | one | +---+ two  |     
     |      |     |     |     |      |     
     +---+--+     +--+--+     +------+     
         |           |                     
   +-----+           +------+              
   |                 |      |              
   | +------+        |      | +------+     
   | |      |        |      | |      |     
   | | 0000 +--------+      +-+ four |     
   | |      |                 |      |     
   | +------+                 +------+     
   |                                       
   +-----+                                 
         |                                 
     +---+--+                              
     |      |                              
     | oooo |                              
     |      |                              
     +------+                              
                                           
                                           
                                           "#
            .to_vec()
        );
    }
}
