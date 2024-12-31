use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::grid::{Coord, Grid};

const DIRECTION_KEY_LEVELS_SILVER: usize = 2;
const DIRECTION_KEY_LEVELS_GOLD: usize = 25;

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
type CostLookup = HashMap<(Action, Action), u64>;

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
    let (optimal_pairs, lookup) = construct_arrow_lookup(&mut errors, direction_start, &direction_grid);
    let cost_lookup = construct_layer_lookups(&lookup, DIRECTION_KEY_LEVELS_SILVER);
    let (silver_solutions, silver) = run_expansion(&numpad_grid, numpad_start, &cost_lookup, inputs.clone(), &mut errors);
    let cost_lookup_gold = construct_layer_lookups(&lookup, DIRECTION_KEY_LEVELS_GOLD);
    let (gold_solutions, gold) = run_expansion(&numpad_grid, numpad_start, &cost_lookup_gold, inputs, &mut errors);

    tabs.push(Tab {
        title: "Errors".to_string(),
        strings: errors,
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Optimal".to_string(),
        strings: optimal_pairs,
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Lookup".to_string(),
        strings: lookup.iter().map(|(pair, actions)| {
            format!("{:?}, {:?}", pair, actions)
        }).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Silver solutions".to_string(),
        strings: silver_solutions.into_iter().map(|(code, cost)| format!("{}: {}", code, cost)).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Gold solutions".to_string(),
        strings: gold_solutions.into_iter().map(|(code, cost)| format!("{}: {}", code, cost)).collect(),
        grid: vec![],
    });

    DayOutput {
        silver_output: format!("{}", silver),
        gold_output: format!("{}", gold),
        diagnostic: Diagnostic::with_tabs(tabs, String::new()),
    }
}

fn run_expansion(numpad_grid: &Grid<NumpadKey>, numpad_start: Coord, cost_lookup: &CostLookup, inputs: Vec<(Vec<NumpadKey>, u64)>, errors: &mut Vec<String>) -> (Vec<(u64, u64)>, u64) {
    let mut response_value = 0;
    let mut separated_scores = Vec::new();
    for (input_row, code) in inputs {
        let mut action_possibilities = reverse_engineer(&numpad_grid, NumpadKey::Empty, numpad_start, &vec![input_row], errors);
        let mut costs = Vec::new();
        for possibility in action_possibilities {
            let cost = find_cost_for_sequence(cost_lookup, &possibility);
            costs.push(cost);
        }
        if let Some(smallest_cost) = costs.iter().min() {
            response_value += code * *smallest_cost as u64;
            separated_scores.push((code, *smallest_cost));
        }
    }
    (separated_scores, response_value)
}

fn construct_arrow_lookup(mut errors: &mut Vec<String>, direction_start: Coord, direction_grid: &Grid<Action>) -> (Vec<String>, HashMap<(Action, Action), Vec<Action>>) {
    let optimal_clusters = Action::get_all_pairs().into_iter().map(|(first, second)| {
        let mut outputs: Vec<(Vec<Action>, Vec<Action>, Vec<Action>, Vec<Action>)> = Vec::new();
        let seqs1 = reverse_engineer_from_first(&direction_grid, &vec![first, second], &mut errors);
        for seq1 in seqs1.into_iter() {
            // errors.push(format!("Seq 1 {}{} {:?}", first, second, seq1));
            let seqs2 = reverse_engineer(&direction_grid, Action::Empty, direction_start, &vec![seq1.clone()], &mut errors);
            for seq2 in seqs2.into_iter() {
                // errors.push(format!("Seq 2 {}{} {:?}", first, second, seq2));
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
    (optimal_pairs, lookup)
}

fn construct_layer_lookups(arrow_lookup: &ArrowLookup, num_layers: usize) -> CostLookup {
    let mut remaining_layers = num_layers - 1;
    let mut previous_layer = construct_first_layer_lookup(arrow_lookup);
    for _layer in (0..remaining_layers).rev() {
        let new_layer = construct_layer_lookup(arrow_lookup, &previous_layer);
        previous_layer = new_layer;
    }
    previous_layer
}
fn construct_layer_lookups_all(arrow_lookup: &ArrowLookup, num_layers: usize) -> HashMap<usize, CostLookup> {
    let mut remaining_layers = num_layers - 1;
    let mut metamap = HashMap::new();
    let mut previous_layer = construct_first_layer_lookup(arrow_lookup);
    metamap.insert(remaining_layers, previous_layer.clone());
    for layer in (0..remaining_layers).rev() {
        let new_layer = construct_layer_lookup(arrow_lookup, &previous_layer);
        metamap.insert(layer, new_layer.clone());
        previous_layer = new_layer;
    }
    metamap
}

fn construct_layer_lookup(arrow_lookup: &ArrowLookup, sub_layer_lookup: &CostLookup) -> CostLookup {
    let mut output = HashMap::new();
    for pair in Action::get_all_pairs() {
        if let Some(sequence) = arrow_lookup.get(&pair) {
            let total_cost = find_cost_for_sequence(sub_layer_lookup, &sequence);
            output.insert(pair, total_cost);
        }
    }
    output
}

fn construct_first_layer_lookup(arrow_lookup: &ArrowLookup) -> CostLookup {
    HashMap::from_iter(arrow_lookup.iter().map(|(pair, list)| (pair.clone(), list.len() as u64)))
}

fn find_cost_for_sequence(cost_lookup: &CostLookup, sequence: &Vec<Action>) -> u64 {
    let mut last = Action::A;
    let mut total_cost = 0;
    for item in sequence {
        let current_pair = (last, *item);
        if let Some(cost) = cost_lookup.get(&current_pair) {
            total_cost += cost;
        }
        last = *item;
    }
    total_cost
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
    let all_output_sequences = all_output_sequences.into_iter().filter(|sequence| {
        playback_ok(grid, blank.clone(), start_pos, sequence)
    }).collect::<Vec<_>>();
    all_output_sequences
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
