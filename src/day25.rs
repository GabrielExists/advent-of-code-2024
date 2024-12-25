use crate::app::{DayOutput, Diagnostic, Tab};

type Shape = Vec<u8>;

#[derive(Clone, Debug)]
enum Block {
    Key(Shape),
    Lock(Shape),
    #[allow(dead_code)]
    Error(String),
}

pub fn puzzle(input: &str) -> DayOutput {
    let errors: Vec<String> = Vec::new();
    let mut tabs: Vec<Tab> = Vec::new();

    let blocks = input.split("\n\n").map(|block| {
        let mut split = block.split("\n");
        let first = split.next();
        let middle = split.clone().take(5).collect::<Vec<_>>();
        let last = split.skip(5).next();
        match (first, last) {
            (Some("#####"), Some(".....")) => {
                parse_block(middle, Block::Lock)
            }
            (Some("....."), Some("#####")) => {
                parse_block(middle, Block::Key)
            }
            (first, last) => Block::Error(format!("{:?}, {:?}", first, last)),
        }
    }).collect::<Vec<Block>>();
    let locks = blocks.iter().filter_map(|block| {
        match block {
            Block::Key(_) => None,
            Block::Lock(lock) => Some(lock.clone()),
            Block::Error(_) => None,
        }
    }).collect::<Vec<_>>();

    let keys = blocks.iter().filter_map(|block| {
        match block {
            Block::Key(key) => Some(key.clone()),
            Block::Lock(_) => None,
            Block::Error(_) => None,
        }
    }).collect::<Vec<_>>();

    let mut silver = 0;
    let overlaps = Vec::new();
    for lock in locks.iter() {
        for key in keys.iter() {
            let compatible = check_compatible_overlap(lock, key);
            if compatible {
                silver += 1;
            }
            // overlaps.push(format!("{:?} + {:?} => {}", lock, key, compatible));
        }
    }

    tabs.push(Tab {
        title: "Input".to_string(),
        strings: blocks.iter().map(|block| format!("{:?}", block)).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Locks".to_string(),
        strings: locks.iter().map(|block| format!("{:?}", block)).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Keys".to_string(),
        strings: keys.iter().map(|block| format!("{:?}", block)).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Overlaps".to_string(),
        strings: overlaps,
        grid: vec![],
    });

    DayOutput {
        silver_output: format!("{}", silver),
        gold_output: format!("{}", 0),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
    }
}

fn parse_block<F>(lines: Vec<&str>, wrap: F) -> Block
    where F: Fn(Shape) -> Block {
    let mut shape = vec![0, 0, 0, 0, 0];

    for line in lines {
        for (index, character) in line.chars().enumerate() {
            if let Some(entry) = shape.get_mut(index) {
                match character {
                    '#' => {
                        *entry += 1;
                    }
                    '.' => {}
                    error_character => return Block::Error(format!("Broken character {}", error_character)),
                }
            } else {
                return Block::Error(format!("Index {} not available", index));
            }
        }
    }
    wrap(shape)
}

fn check_compatible_overlap(lock: &Shape, key: &Shape) -> bool {
    for (lock_height, key_height) in std::iter::zip(lock.iter(), key.iter()) {
        if *lock_height + *key_height > 5 {
            return false;
        }
    }
    return true;
}