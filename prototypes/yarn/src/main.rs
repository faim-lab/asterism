use asterism::{
    linking::GraphedLinking,
    resources::{PoolInfo, QueuedResources, ResourceError, Transaction},
};
use rand::prelude::*;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
enum PoolID {
    Energy,
    NumSleep,
}

impl PoolInfo for PoolID {
    fn min_value(&self) -> f64 {
        0.0
    }

    fn max_value(&self) -> f64 {
        match self {
            Self::Energy => 100.0,
            Self::NumSleep => 3.0,
        }
    }
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
            println!("{}", error);
            return;
        }
        Ok(..) => {}
    };

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
                    .push(vec![(PoolID::NumSleep, Transaction::Change(1.0))]);
                logics
                    .resources
                    .transactions
                    .push(vec![(PoolID::Energy, Transaction::Change(12.0))]);
            }
            7 => {
                logics
                    .resources
                    .transactions
                    .push(vec![(PoolID::Energy, Transaction::Change(6.0))]);
            }
            8 => {
                logics.resources.transactions.push(vec![(
                    PoolID::Energy,
                    Transaction::Change(rng.gen_range(11.0, 17.0)),
                )]);
            }
            9 => {
                logics.resources.transactions.push(vec![(
                    PoolID::Energy,
                    Transaction::Change(rng.gen_range(10.0, 20.0)),
                )]);
            }
            10 => {
                logics.resources.transactions.push(vec![(
                    PoolID::Energy,
                    Transaction::Change(-rng.gen_range(15.0, 20.0)),
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
            self.current_label = *position;
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
        for completed in resources.completed.iter() {
            match completed {
                Ok(item_types) => {
                    for item_type in item_types {
                        let value = resources
                            .get_value_by_itemtype(item_type)
                            .unwrap()
                            .min(item_type.max_value()) as u8;
                        match item_type {
                            PoolID::NumSleep => self.numsleep = value,
                            PoolID::Energy => self.energy = value,
                        }
                    }
                }
                Err(err) => match err {
                    ResourceError::TooSmall(pool) => match pool {
                        PoolID::Energy => self.energy = 0,
                        _ => {}
                    },
                    ResourceError::TooBig(pool) => match pool {
                        PoolID::Energy => self.energy = 100,
                        _ => {}
                    },
                    _ => {}
                },
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

fn read_file(world: &mut World, linking: &mut GraphedLinking) -> Result<(), &'static str> {
    let text = include_str!("text");
    let mut current_label;
    let mut labels: Vec<String> = Vec::new();
    let mut links: Vec<(usize, String)> = Vec::new();
    let mut link_count: u8 = 0;
    let mut nodes: Vec<Vec<usize>> = Vec::new();

    for line in text.lines() {
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
            return Err("error reading file");
        }
        nodes[from_label].push(idx as usize);
    }

    linking.add_link_map(0, nodes);
    Ok(())
}
