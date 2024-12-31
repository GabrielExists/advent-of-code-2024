use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::grid::{Coord, Grid};

const DIRECTION_KEY_LEVELS_SILVER: usize = 2;
const DIRECTION_KEY_LEVELS_GOLD: usize = 15;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Action {
    A,
    Up,
    Down,
    Left,
    Right,
    Empty,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum NumpadKey {
    Number(u8),
    A,
    Empty,
}

type ArrowLookup = HashMap<(Action, Action), Vec<Action>>;

pub fn puzzle(input: &str) -> DayOutput {
    // Parse input
    let mut errors: Vec<String> = Vec::new();
    let inputs = input.split("\n").into_iter().filter_map(|line| {
        let code = line.trim_matches('A').parse::<u64>().ok();
        let line = line.chars().filter_map(|char| {
            match char {
                'A' => Some(NumpadKey::A),
                char => {
                    char.to_digit(10).map(|dig| NumpadKey::Number(dig as u8))
                }
            }
        }).collect::<Vec<_>>();
        if let (Some(code), false) = (code, line.is_empty()) {
            Some((line, code))
        } else {
            None
        }
    }).collect::<Vec<(Vec<NumpadKey>, u64)>>();

    let mut tabs: Vec<Tab> = Vec::new();
    let numpad_start = Coord::new(2, 3);
    let numpad_grid = Grid(vec![
        vec![NumpadKey::Number(7), NumpadKey::Number(8), NumpadKey::Number(9)],
        vec![NumpadKey::Number(4), NumpadKey::Number(5), NumpadKey::Number(6)],
        vec![NumpadKey::Number(1), NumpadKey::Number(2), NumpadKey::Number(3)],
        vec![NumpadKey::Empty, NumpadKey::Number(0), NumpadKey::A],
    ]);
    let direction_start = Coord::new(2, 0);
    let direction_grid = Grid(vec![
        vec![Action::Empty, Action::Up, Action::A],
        vec![Action::Left, Action::Down, Action::Right],
    ]);

    // Construct lookup for direction grid
    let optimal_clusters = Action::get_all_pairs().into_iter().map(|(first, second)| {
        let mut outputs: Vec<(Vec<Action>, Vec<Action>, Vec<Action>, Vec<Action>)> = Vec::new();
        let seqs1 = reverse_engineer_from_first(&direction_grid, &vec![first, second], &mut errors);
        for seq1 in seqs1.into_iter() {
            errors.push(format!("Seq 1 {}{} {:?}", first, second, seq1));
            let seqs2 = reverse_engineer(&direction_grid, Action::Empty, direction_start, &vec![seq1.clone()], &mut errors);
            for seq2 in seqs2.into_iter() {
                errors.push(format!("Seq 2 {}{} {:?}", first, second, seq2));
                let seqs3 = reverse_engineer(&direction_grid, Action::Empty, direction_start, &vec![seq2.clone()], &mut errors);
                for seq3 in seqs3.into_iter() {
                    let seqs4 = reverse_engineer(&direction_grid, Action::Empty, direction_start, &vec![seq3.clone()], &mut errors);
                    for seq4 in seqs4.into_iter() {
                        outputs.push((seq1.clone(), seq2.clone(), seq3.clone(), seq4.clone()))
                    }
                }
            }
        }
        let shortest = outputs.into_iter().min_by_key(|(_, _, _, seq4)| {
            seq4.len()
        });
        ((first, second), shortest)
    }).collect::<Vec<_>>();
    let optimal_pairs = optimal_clusters.iter().map(|(pair, lists)| {
        if let Some((seq1, seq2, _seq3, seq4)) = lists {
            format!("{} to {}, {}: {:?} | {:?}", pair.0, pair.1, seq4.len(), seq1, seq2)
        } else {
            String::new()
        }
    }).collect::<Vec<_>>();
    let mut lookup = HashMap::new();
    for (pair, lists) in optimal_clusters.into_iter() {
        if let Some((seq1, _, _, _)) = lists {
            lookup.insert(pair, seq1);
        } else {
            errors.push(format!("Pair {:?} gave a None", pair));
        }
    }
    // >>>>A
    // v<<A

    // Process each row
    let (output_gold, gold) = run_expansion(&mut errors, inputs.clone(), numpad_start, &numpad_grid, &lookup, DIRECTION_KEY_LEVELS_GOLD);
    // let gold = 0;
    let (expansion_log, possibilities, outputs_silver, stepped_outputs, silver) = run_expansion_logged(&mut errors, inputs.clone(), numpad_start, &numpad_grid, &lookup, DIRECTION_KEY_LEVELS_SILVER);
    tabs.push(Tab {
        title: "Output Gold".to_string(),
        strings: output_gold.into_iter().map(|sequence|{
            format!("{:?}", sequence)
        }).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Errors".to_string(),
        strings: errors,
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Outputs".to_string(),
        strings: outputs_silver.iter().map(|(_actions, code, length)| format!("Length {}, Code {}", length, code)).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Stepped outputs".to_string(),
        strings: stepped_outputs,
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Optimal".to_string(),
        strings: optimal_pairs,
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Possibilities".to_string(),
        strings: possibilities,
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Expansion log".to_string(),
        strings: expansion_log.into_iter().map(|list| format!("{:?}", list)).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Lookup".to_string(),
        strings: lookup.iter().map(|(pair, actions)| {
            format!("{:?}, {:?}", pair, actions)
        }).collect(),
        grid: vec![],
    });

    DayOutput {
        silver_output: format!("{}", silver),
        gold_output: format!("{}", gold),
        diagnostic: Diagnostic::with_tabs(tabs, String::new()),
    }
}

fn run_expansion(errors: &mut Vec<String>, inputs: Vec<(Vec<NumpadKey>, u64)>, numpad_start: Coord, numpad_grid: &Grid<NumpadKey>, lookup: &ArrowLookup, num_arrow_expansions: usize) -> (Vec<Vec<Action>>, u64) {
    let mut response_value = 0;
    let mut all_shortest = Vec::new();
    for (input_row, code) in inputs {
        let mut action_possibilities = reverse_engineer(&numpad_grid, NumpadKey::Empty, numpad_start, &vec![input_row], errors);
        let mut expanded_possibilities = Vec::new();
        for input_actions in action_possibilities.iter() {
            let mut actions = input_actions.clone();
            for _level in 0..num_arrow_expansions {
                actions = expand_arrow_key(&lookup, &actions, errors);
            }
            expanded_possibilities.push(actions);
        }
        let shortest = expanded_possibilities.into_iter().min_by_key(|sequence| sequence.len()).unwrap_or(Vec::new());
        let length = shortest.len();
        response_value += code * length as u64;
        all_shortest.push(shortest);
    }
    (all_shortest, response_value)
}

fn run_expansion_logged(mut errors: &mut Vec<String>, inputs: Vec<(Vec<NumpadKey>, u64)>, numpad_start: Coord, numpad_grid: &Grid<NumpadKey>, lookup: &ArrowLookup, num_arrow_expansions: usize) -> (Vec<Vec<Action>>, Vec<String>, Vec<(Vec<Action>, u64, usize)>, Vec<String>, u64) {
    let mut expansion_log = Vec::new();
    let mut possibilities = Vec::new();
    let mut outputs_silver = Vec::new();
    let mut stepped_outputs = Vec::new();
    let mut response_value = 0;
    for (input_row, code) in inputs {
        let mut action_possibilities = reverse_engineer(&numpad_grid, NumpadKey::Empty, numpad_start, &vec![input_row], errors);
        let mut expanded_possibilities = Vec::new();
        for input_actions in action_possibilities.iter() {
            let mut actions = input_actions.clone();
            stepped_outputs.push(format!("{:?}", actions));
            expansion_log.push(actions.clone());
            for _level in 0..num_arrow_expansions {
                actions = expand_arrow_key(&lookup, &actions, &mut errors);
                // expansion_log.push(actions.clone());

                stepped_outputs.push(format!("{:?}", actions));
                // actions = reverse_engineer(&direction_grid, Action::Empty, direction_start, &actions, &mut errors);
            }
            expanded_possibilities.push(actions);
        }
        // errors.push(format!("{:?}", action_possibilities));
        let num_possibilities = expanded_possibilities.len();
        let longest_len = expanded_possibilities.iter().map(|list| list.len()).min().unwrap_or(0);
        let shortest = expanded_possibilities.into_iter().min_by_key(|sequence| sequence.len()).unwrap_or(Vec::new());
        let shortest_len = shortest.len();
        possibilities.push(format!("{:03}A: Silver had {} possibilties, longest: {}, shortest: {}", code, num_possibilities, longest_len, shortest_len));
        // stepped_outputs.push(format!("{:?}", shortest));
        let length = shortest.len();
        response_value += code * length as u64;
        outputs_silver.push((shortest, code, length));
    }
    (expansion_log, possibilities, outputs_silver, stepped_outputs, response_value)
}

pub fn reverse_engineer_from_first(grid: &Grid<Action>, sequences: &Vec<Action>, errors: &mut Vec<String>) -> Vec<Vec<Action>> {
    if let Some((first, rest)) = sequences.split_first() {
        if let Some(start) = grid.find(|item| *item == *first) {
            reverse_engineer(&grid, Action::Empty, start, &vec![rest.to_vec()], errors)
        } else {
            errors.push(format!("Found no item {} in grid", first));
            Vec::new()
        }
    } else {
        errors.push(format!("List {:?} could not be split", sequences));
        Vec::new()
    }
}

pub fn reverse_engineer<T>(grid: &Grid<T>, blank: T, start_pos: Coord, sequences: &Vec<Vec<T>>, errors: &mut Vec<String>) -> Vec<Vec<Action>>
    where T: Eq + Hash + Clone {
    let mut coordinate_lookup: HashMap<T, Coord> = HashMap::new();
    let _ = grid.map_grid(|cell, x, y| {
        coordinate_lookup.entry(cell.clone()).or_insert(Coord((x as i32, y as i32)));
    });
    let mut all_output_sequences = Vec::new();
    let mut current_pos = start_pos;
    for sequence in sequences.iter() {
        let mut output_sequences = vec![vec![]];
        for item in sequence.iter() {
            if let Some(new_pos) = coordinate_lookup.get(item) {
                let delta = new_pos.subtract(&current_pos);
                let Coord((delta_x, delta_y)) = delta;
                let mut vertical = vec![];
                let mut horizontal = vec![];
                if delta_y > 0 {
                    for _ in 0..delta_y {
                        vertical.push(Action::Down);
                    }
                }
                if delta_y < 0 {
                    for _ in 0..-delta_y {
                        vertical.push(Action::Up);
                    }
                }
                if delta_x > 0 {
                    for _ in 0..delta_x {
                        horizontal.push(Action::Right);
                    }
                }
                if delta_x < 0 {
                    for _ in 0..-delta_x {
                        horizontal.push(Action::Left);
                    }
                }
                if !vertical.is_empty() && !horizontal.is_empty() {
                    let order_one = horizontal.clone().into_iter().chain(vertical.clone().into_iter()).chain([Action::A].into_iter()).collect::<Vec<_>>();
                    let order_two = vertical.clone().into_iter().chain(horizontal.clone().into_iter()).chain([Action::A].into_iter()).collect::<Vec<_>>();
                    output_sequences = add_possibilities(output_sequences, vec![order_one, order_two], errors);
                } else {
                    let order_one = horizontal.clone().into_iter().chain(vertical.clone().into_iter()).chain([Action::A].into_iter()).collect::<Vec<_>>();
                    output_sequences = add_possibilities(output_sequences, vec![order_one], errors);
                }
                current_pos = *new_pos;
            }
        }
        all_output_sequences.append(&mut output_sequences);
    }
    // let output_sequence = output_sequences.into_iter().min_by_key(|sequence| sequence.len());
    // output_sequence.unwrap_or(vec![])
    let possibilities_before = all_output_sequences.len();
    let min_length = all_output_sequences.iter().map(|sequence| sequence.len()).min().unwrap_or(0);
    let all_output_sequences = all_output_sequences.into_iter().filter(|sequence| {
        // if sequence.len() > min_length {
        //     return false;
        // }
        playback_ok(grid, blank.clone(), start_pos, sequence)
        // true
    }).collect::<Vec<_>>();
    // errors.push(format!("Pruning from {} to {}", possibilities_before, all_output_sequences.len()));
    all_output_sequences
}

fn expand_arrow_key(lookup: &ArrowLookup, sequence: &Vec<Action>, errors: &mut Vec<String>) -> Vec<Action> {
    let mut output = Vec::new();
    let mut last = Action::A;
    for item in sequence {
        let pair = (last, *item);
        match lookup.get(&pair) {
            Some(addition) => {
                output.extend(addition.iter());
            }
            None => {
                errors.push(format!("Couldn't find pair {:?}", pair));
                return Vec::new();
            }
        }
        last = *item;
    }
    output
}

fn add_possibilities(sequences: Vec<Vec<Action>>, possibilities: Vec<Vec<Action>>, errors: &mut Vec<String>) -> Vec<Vec<Action>> {
    errors.push(format!("adding possibilities {:?}", possibilities));
    let mut output_sequences = Vec::new();
    for possibility in possibilities.into_iter() {
        for sequence in sequences.iter() {
            let mut sequence = sequence.clone();
            sequence.extend(possibility.iter());
            output_sequences.push(sequence);
        }
    }
    output_sequences
}

fn playback_ok<T>(grid: &Grid<T>, blank: T, start_pos: Coord, sequence: &Vec<Action>) -> bool
    where T: Eq + Hash + Clone {
    if let Some(tile) = grid.get(start_pos) {
        if *tile == blank {
            return false;
        }
    } else {
        return false;
    }
    let mut pos = start_pos;
    for action in sequence.iter() {
        match action {
            Action::Up => {
                pos = pos.add(&Coord::new(0, -1));
            }
            Action::Down => {
                pos = pos.add(&Coord::new(0, 1));
            }
            Action::Left => {
                pos = pos.add(&Coord::new(-1, 0));
            }
            Action::Right => {
                pos = pos.add(&Coord::new(1, 0));
            }
            Action::A => {}
            Action::Empty => {
                return false;
            }
        }
        if let Some(tile) = grid.get(pos) {
            if *tile == blank {
                return false;
            }
        } else {
            return false;
        }
    }
    true
}

impl Action {
    pub fn get_all() -> Vec<Action> {
        vec![
            Action::A,
            Action::Up,
            Action::Down,
            Action::Left,
            Action::Right,
        ]
    }
    pub fn get_all_pairs() -> Vec<(Action, Action)> {
        Self::get_all().into_iter().map(|first| {
            Self::get_all().into_iter().map(move |second| {
                (first, second)
            })
        }).flatten().collect()
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::A => f.write_str("A"),
            Action::Up => f.write_str("^"),
            Action::Down => f.write_str("v"),
            Action::Left => f.write_str("<"),
            Action::Right => f.write_str(">"),
            Action::Empty => f.write_str("!"),
        }
    }
}

// +---+---+---+
// | 7 | 8 | 9 |
// +---+---+---+
// | 4 | 5 | 6 |
// +---+---+---+
// | 1 | 2 | 3 |
// +---+---+---+
//     | 0 | A |
//     +---+---+

//     +---+---+
//     | ^ | A |
// +---+---+---+
// | < | v | > |
// +---+---+---+
