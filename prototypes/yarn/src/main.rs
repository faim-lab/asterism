use asterism::{linking::GraphedLinking, resources::QueuedResources, resources::Transaction};
use rand::prelude::*;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, Error, ErrorKind};

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
enum PoolID {
    Energy,
    NumSleep,
}

struct World {
    dialogue: Vec<Vec<String>>,
    num_links: Vec<u8>,
    current_label: usize,
    choice: Option<usize>,
    numsleep: u8,
    energy: u8,
}

struct Logics {
    resources: QueuedResources<PoolID>,
    linking: GraphedLinking,
}

impl Logics {
    fn new() -> Self {
        Self {
            resources: {
                let mut resources = QueuedResources::new();
                resources.items.insert(PoolID::Energy, 49.0);
                resources.items.insert(PoolID::NumSleep, 0.0);
                resources
            },
            linking: GraphedLinking::new(),
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

    while world.current_label != 11 {
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
            energy: 49,
            numsleep: 0,
        }
    }

    fn update(&mut self, logics: &mut Logics) {
        self.choice = None;
        self.update_energy();
        for line in self.dialogue[self.current_label].iter() {
            println!("{}", line);
        }
        loop {
            let mut choice = String::new();
            let _ = std::io::stdin().read_line(&mut choice).unwrap();
            if let Ok(choice) = choice.trim().parse::<usize>() {
                if choice <= self.num_links[self.current_label] as usize && choice > 0 {
                    self.choice = Some(choice - 1);
                    break;
                }
            }
            println!("enter a valid input please\n");
        }

        self.project_linking(&mut logics.linking);
        logics.linking.update();
        self.unproject_linking(&logics.linking);

        let mut rng = thread_rng();
        match self.current_label {
            0 => {
                self.reset(logics);
            }
            5 => {
                logics
                    .resources
                    .transactions
                    .push(vec![(PoolID::NumSleep, Transaction::Change(1))]);
                logics
                    .resources
                    .transactions
                    .push(vec![(PoolID::Energy, Transaction::Change(12))]);
            }
            7 => {
                logics
                    .resources
                    .transactions
                    .push(vec![(PoolID::Energy, Transaction::Change(-6))]);
            }
            8 => {
                logics.resources.transactions.push(vec![(
                    PoolID::Energy,
                    Transaction::Change(rng.gen_range(11, 17)),
                )]);
            }
            9 => {
                logics.resources.transactions.push(vec![(
                    PoolID::Energy,
                    Transaction::Change(rng.gen_range(10, 20)),
                )]);
            }
            10 => {
                logics.resources.transactions.push(vec![(
                    PoolID::Energy,
                    Transaction::Change(-rng.gen_range(15, 20)),
                )]);
            }
            _ => {}
        }

        self.project_resources(&mut logics.resources);
        logics.resources.update();
        self.unproject_resources(&logics.resources);

        println!();
    }

    fn project_linking(&self, linking: &mut GraphedLinking) {
        if let Some(choice) = self.choice {
            let mut next_label = linking.maps[0].nodes[self.current_label].links[choice];
            match (self.current_label, choice) {
                (2, 0) => {
                    if self.numsleep >= 3 {
                        next_label = 6;
                    }
                }
                (2, 3) => {
                    let mut rng = thread_rng();
                    if rng.gen_range(0, 2) != 0 {
                        next_label = 10;
                    }
                }
                (5..=10, 0) => {
                    if self.energy <= 0 {
                        next_label = 3;
                    } else if self.energy > 99 {
                        next_label = 4;
                    }
                }
                _ => {}
            }
            linking.conditions[0][next_label] = true;
        }
    }

    fn unproject_linking(&mut self, linking: &GraphedLinking) {
        for (_, position) in linking.maps.iter().zip(linking.positions.iter()) {
            if let Some(pos) = position {
                self.current_label = *pos;
            }
        }
    }

    fn project_resources(&self, resources: &mut QueuedResources<PoolID>) {
        if !resources.items.contains_key(&PoolID::NumSleep) {
            resources.items.insert(PoolID::NumSleep, 0.0);
        }
        if !resources.items.contains_key(&PoolID::Energy) {
            resources.items.insert(PoolID::Energy, 49.0);
        }
    }

    fn unproject_resources(&mut self, resources: &QueuedResources<PoolID>) {
        for (completed, item_types) in resources.completed.iter() {
            if *completed {
                for item_type in item_types {
                    let value = resources.get_value_by_itemtype(item_type).min(255.0) as u8;
                    match item_type {
                        PoolID::NumSleep => self.numsleep = value,
                        PoolID::Energy => self.energy = value,
                    }
                }
            } else {
                let last_item_idx = item_types.len() - 1;
                match item_types[last_item_idx] {
                    PoolID::Energy => self.energy = 0,
                    _ => {}
                }
            }
        }
    }

    fn reset(&mut self, logics: &mut Logics) {
        self.energy = 49;
        self.numsleep = 0;
        logics.resources = {
            let mut resources = QueuedResources::new();
            resources.items.insert(PoolID::Energy, 49.0);
            resources.items.insert(PoolID::NumSleep, 0.0);
            resources
        };
    }

    fn update_energy(&mut self) {
        let energy_line_number = self.dialogue[2].len() - 2;
        let mut line = String::from("energy: ");
        line.push_str(&self.energy.to_string());
        line.push_str("%");
        self.dialogue[2][energy_line_number] = line;
    }
}

fn read_file(world: &mut World, linking: &mut GraphedLinking) -> io::Result<()> {
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
                let line = line.trim();
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
                            world.dialogue[labels.len() - 1].push(String::from(
                                link_count.to_string() + &link_label[label_end..],
                            ));
                            links.push((labels.len() - 1, String::from(&line[5..(label_end + 5)])));
                        } else {
                            world.dialogue[labels.len() - 1]
                                .push(String::from(link_count.to_string() + &link_label));
                            links.push((labels.len() - 1, String::from(&line[5..])));
                        }
                    } else if line.starts_with("secretlink") {
                        links.push((labels.len() - 1, String::from(&line[11..])));
                    } else {
                        world.dialogue[labels.len() - 1].push(String::from(line));
                    }
                }
            }
            Err(error) => {
                return Err(error);
            }
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

    linking.add_link_map(Some(0), nodes);

    Ok(())
}
