use std::collections::{BTreeSet, HashMap, HashSet};
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::PathBuf;

use serde::Deserialize;
use structopt::StructOpt;

use ascii_diagrams::{render, Block, LogicalCoord, RenderOptions};

macro_rules! die {
    ( $ ( $ args : tt ) * ) => {
        if cfg!(test) {
            panic!($($args)*);
        } else {
            eprintln!($($args)*);
            std::process::exit(1);
        }
    };
}

macro_rules! try_or_die {
    ($ expr : expr) => {
        match $expr {
            Ok(o) => o,
            Err(e) => die!("{}", e),
        }
    };
}

// serde doesn't support literals as default values yet, have to use functions instead...
//
// See https://github.com/serde-rs/serde/issues/368
const fn default_hmargin() -> usize {
    5
}
const fn default_vmargin() -> usize {
    3
}
const fn default_padding() -> usize {
    1
}

/// Render a diagram using only ASCII characters.
///
/// This is useful to embed diagrams directly as text instead of using images.
///
/// The diagram can be expressed in either TOML or JSON, but the underlying structure is the same.
///
/// Here's an example JSON diagram that shows how to render a very simple diagram.
/// ```json
/// {
///   "blocks": [
///     {
///       "text": "zero",
///       "position": { "column": -1, "row": -1 }
///     },
///     {
///       "text": "one",
///       "position": { "column": 0, "row": -1 }
///     },
///     {
///       "text": "two",
///       "position": { "column": 1, "row": -1 }
///     },
///     {
///       "text": "0000",
///       "position": { "column": -1, "row": 0 }
///     },
///     {
///       "text": "four",
///       "position": { "column": 1, "row": 0 }
///     },
///     {
///       "text": "oooo",
///       "position": { "column": -1, "row": 1 }
///     }
///   ],
///   "edges": [
///     { "from": "one", "to": "four" },
///     { "from": "one", "to": "0000" },
///     { "from": "two", "to": "zero" },
///     { "from": "oooo", "to": "zero" }
///   ]
/// }
/// ```
///
/// I also think these diagrams are quite neat to look at.
#[derive(Debug, StructOpt)]
struct Opts {
    /// Input diagram to render in either TOML or JSON.
    #[structopt(name = "INPUT", parse(from_os_str))]
    diagram: PathBuf,

    /// Output file where to save the final ascii diagram. If nothing is passed stdout will be
    /// used.
    #[structopt(name = "OUTPUT", parse(from_os_str))]
    output: Option<PathBuf>,

    /// Seed to use for the rendering algorithm.
    #[structopt(long)]
    seed: Option<u64>,

    /// Maximum number of tweaks to find the best arrangement of lines.
    #[structopt(long, default_value = "100")]
    max_tweaks: usize,
}

#[derive(Deserialize)]
struct Spec {
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

    let mut f = try_or_die!(File::open(&opts.diagram));
    let mut input_spec = vec![];
    try_or_die!(f.read_to_end(&mut input_spec));

    let spec: Spec = match opts
        .diagram
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
    {
        "json" => try_or_die!(serde_json::from_slice(&input_spec)),
        "toml" => try_or_die!(toml::from_slice(&input_spec)),
        e => {
            die!(
                r#"unrecognized diagram format "{}", valid extensions: toml, json"#,
                e
            );
        }
    };

    let canvas = render_diagram(spec, &opts);

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

fn render_diagram(spec: Spec, opts: &Opts) -> Vec<Vec<u8>> {
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
            seed: opts.seed,
            max_tweaks: opts.max_tweaks,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use ascii_diagrams::assert_diagram_eq;

    #[test]
    fn test_basic_toml_diagram() {
        let diagram = br#"
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
        let diagram = toml::from_slice(diagram).unwrap();

        assert_diagram_eq!(
            render_diagram(
                diagram,
                &Opts {
                    diagram: PathBuf::new(),
                    output: None,
                    seed: Some(42),
                    max_tweaks: 0,
                }
            ),
            br#"                                           
         +------------------------+        
         |                        |        
     +---+--+     +-----+     +---+--+     
     |      |     |     |     |      |     
   +-+ zero |     | one +-+   | two  |     
   | |      |     |     | |   |      |     
   | +------+     +--+--+ |   +------+     
   |                 |    |                
   |                 |    +-------+        
   |                 |            |        
   | +------+        |        +---+--+     
   | |      |        |        |      |     
   | | 0000 +--------+        | four |     
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
}
