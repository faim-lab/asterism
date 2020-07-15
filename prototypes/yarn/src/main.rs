use asterism::{/* Resources, resources::Transaction,*/ Linking};
use std::io::{self, prelude::*, BufReader, Error, ErrorKind};
use std::fs::File;

/* #[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
struct PoolID {
} */

struct World {
    dialogue: Vec<Vec<String>>,
    num_links: Vec<u8>,
    current_label: usize,
    choice: Option<usize>,
}


struct Logics {
    // resources: Resources<PoolID>,
    linking: Linking,
}

impl Logics {
    fn new() -> Self {
        Self {
            // resources: Resources::new(),
            linking: Linking::new()
        }
    }
}

fn main() {
    let mut logics = Logics::new();
    let mut world = World::new();
    match read_file(&mut world, &mut logics.linking) {
        Err(error) => {
            println!("{:?}", error);
            return;
        }
        Ok(..) => {}
    }

    loop {
        world.update(&mut logics);
    }
}

impl World {
    fn new() -> Self {
        Self {
            dialogue: Vec::new(),
            num_links: Vec::new(),
            current_label: 0,
            choice: None,
        }
    }

    fn update(&mut self, logics: &mut Logics) {
        self.choice = None;
        for line in self.dialogue[self.current_label].iter() {
            println!("{}", line);
        }
        loop {
            let mut choice = String::new();
            let _ = std::io::stdin().read_line(&mut choice).unwrap();
            if let Ok(choice) = choice.trim().parse::<usize>() {
                if choice <= self.num_links[self.current_label] as usize {
                    self.choice = Some(choice - 1);
                    break;
                }
            }
            println!("enter a valid input or else");
        }

        self.project_linking(&mut logics.linking);
        logics.linking.update();
        self.unproject_linking(&logics.linking);
    }

    fn project_linking(&self, linking: &mut Linking) {
        if let Some(choice) = self.choice {
            linking.conditions[0][linking.maps[0].nodes[self.current_label].links[choice]] = true;
        }
    }

    fn unproject_linking(&mut self, linking: &Linking) {
        for (.., pos) in linking.maps.iter().zip(linking.positions.iter()) {
            self.current_label = *pos;
        }
    }
}

fn read_file(world: &mut World, linking: &mut Linking) -> io::Result<()> {
    let f = File::open("text")?;
    let f = BufReader::new(f);
    let mut current_label;
    let mut labels: Vec<String> = Vec::new();
    let mut links: Vec<(usize, String)> = Vec::new();
    let mut link_count: u8 = 0;
    let mut nodes: Vec<Vec<usize>> = Vec::new();

    for line in f.lines() {
        match line {
            Ok(line) => {
                if line.starts_with("label") {
                    current_label = String::from(&line[6..]);
                    if labels.len() != 0 {
                        world.num_links.push(link_count);
                    }
                    labels.push((*current_label).to_string());
                    world.dialogue.push(Vec::new());
                    link_count = 0;
                } else if labels.len() > 0 {
                    if line.starts_with("link") {
                        link_count += 1;
                        let link_label = String::from(&line[5..]);
                        if let Some(label_end) = link_label.find(" ") {
                            world.dialogue[labels.len() - 1].push(String::from(link_count.to_string() + &link_label[label_end..]));
                            links.push((labels.len() - 1, String::from(&line[5..(label_end + 5)])));
                        } else {
                            world.dialogue[labels.len() - 1].push(String::from(link_count.to_string() + &link_label));
                            links.push((labels.len() - 1, String::from(&line[5..])));
                        }
                    } else if line != "" {
                        world.dialogue[labels.len() - 1].push(line);
                    }
                }
            }
            Err(error) => { return Err(error); }
        }
    }
    world.num_links.push(link_count);

    nodes.resize_with(labels.len(), Default::default);

    for (from_label, to_label) in links {
        let mut idx: i32 = -1;
        for (i, label) in labels.iter().enumerate() {
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
