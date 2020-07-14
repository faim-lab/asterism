#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use asterism::{Resources, resources::Transaction, Linking};
use std::io::{self, prelude::*, Read, BufReader, Error, ErrorKind};
use std::fs::File;
// use std::collections::BTreeMap;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
struct PoolID {
}

struct Dialogue {
    labels: Vec<String>,
    lines: Vec<Vec<String>>
}

struct Logics {
    resources: Resources<PoolID>,
    linking: Linking,
}

impl Logics {
    fn new() -> Self {
        Self {
            resources: Resources::new(),
            linking: Linking::new()
        }
    }
}

fn main() {
    let mut logics = Logics::new();
    let mut d: Dialogue = Dialogue {
        labels: Vec::new(),
        lines: Vec::new(),
    };
    match read_file(&mut d, &mut logics.linking) {
        Err(..) => {
            println!("could not parse file");
            return;
        }
        Ok(..) => {
            for map in logics.linking.maps.iter() {
                for (i, node) in map.nodes.iter().enumerate() {
                    print!("{}: ", i);
                    for link in node.links.iter() {
                        print!("{}", link);
                    }
                    println!();
                }
            }
        }
    }
}

fn read_file(dialogue: &mut Dialogue, linking: &mut Linking) -> io::Result<()> {
    let f = File::open("text")?;
    let f = BufReader::new(f);
    let mut current_label;
    let mut links: Vec<(usize, String)> = Vec::new();
    let mut nodes: Vec<Vec<usize>> = Vec::new();

    for line in f.lines() {
        match line {
            Ok(line) => {
                if line.starts_with("label") {
                    current_label = String::from(&line[6..]);
                    dialogue.labels.push((*current_label).to_string());
                    dialogue.lines.push(Vec::new());
                } else if dialogue.labels.len() > 0 {
                    if line.starts_with("link") {
                        links.push((dialogue.labels.len() - 1, String::from(&line[5..])));
                    } else if line != "" {
                        dialogue.lines[dialogue.labels.len() - 1].push(line);
                    }
                }
            }
            Err(error) => { return Err(error); }
        }
    }

    nodes.resize_with(dialogue.labels.len(), Default::default);

    for (from_label, to_label) in links {
        let mut idx: i32 = -1;
        for (i, label) in dialogue.labels.iter().enumerate() {
            if *label == to_label {
                idx = i as i32;
            }
        }
        if idx < 0 {
            return Err(Error::new(ErrorKind::InvalidData, "error reading file"));
        }
        nodes[from_label].push(idx as usize);
    }

    linking.add_link_map(0, nodes);

    Ok(())
}
