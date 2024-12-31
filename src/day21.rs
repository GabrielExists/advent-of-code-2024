use std::collections::HashMap;
use std::fmt::{Display, Formatter, Write};
use std::hash::Hash;
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::grid::{Coord, Grid};

const DIRECTION_KEY_LEVELS_SILVER: usize = 3;
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

pub fn puzzle(input: &str) -> DayOutput {
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
    let mut outputs = Vec::new();
    let mut stepped_outputs = Vec::new();
    let mut silver = 0;
    for (input_row, code) in inputs {
        let mut actions = reverse_engineer(&numpad_grid, NumpadKey::Empty, numpad_start, &vec![input_row], &mut errors);
        for level in 1..DIRECTION_KEY_LEVELS_SILVER {
            actions = reverse_engineer(&direction_grid, Action::Empty, direction_start, &actions, &mut errors);
            // errors.push(format!("Silver {} had {} possibilties", level, actions.len()));
        }
        let shortest = actions.into_iter().min_by_key(|sequence| sequence.len()).unwrap_or(Vec::new());
        stepped_outputs.push(format!("{:?}", shortest));
        let length = shortest.len();
        silver += code * length as u64;
        outputs.push((shortest, code, length));
    }

    let optimal_pairs = Action::get_all_pairs().into_iter().map(|(first, second)| {
        let mut outputs: Vec<(Vec<Action>, Vec<Action>, Vec<Action>, Vec<Action>)> = Vec::new();
        let seqs1 = reverse_engineer_list(&direction_grid, &vec![first, second], &mut errors);
        for seq1 in seqs1.into_iter() {
            let seqs2 = reverse_engineer_list(&direction_grid, &seq1, &mut errors);
            for seq2 in seqs2.into_iter() {
                let seqs3 = reverse_engineer_list(&direction_grid, &seq2, &mut errors);
                for seq3 in seqs3.into_iter() {
                    let seqs4 = reverse_engineer_list(&direction_grid, &seq3, &mut errors);
                    for seq4 in seqs4.into_iter() {
                        outputs.push((seq1.clone(), seq2.clone(), seq3.clone(), seq4.clone()))
                    }
                }
            }
        }
        let shortest = outputs.into_iter().min_by_key(|(_, _, _, seq4)| {
            seq4.len()
        });
        if let Some((seq1, seq2, seq3, seq4)) = shortest {
            format!("{} to {}, {}: {:?} | {:?}", first, second, seq4.len(), seq1, seq2)
        } else {
            String::new()
        }
    }).collect::<Vec<String>>();

    tabs.push(Tab {
        title: "Optimal".to_string(),
        strings: optimal_pairs,
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Outputs".to_string(),
        strings: outputs.iter().map(|(_actions, code, length)| format!("Length {}, Code {}", length, code)).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Stepped outputs".to_string(),
        strings: stepped_outputs,
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Analysis".to_string(),
        strings: [
            (Action::A, Action::Down),
            (Action::Down, Action::A),
            (Action::Right, Action::Up),
            (Action::Up, Action::Right),
        ].into_iter().map(|(from, to)| {
            let seq1 = get_transition(from, to).unwrap_or(Vec::new());
            let seq2 = expand_numpad(&seq1, &mut errors);
            let seq3 = expand_numpad(&seq2, &mut errors);
            let seq4 = expand_numpad(&seq3, &mut errors);
            [
                format!("{:?}->{:?}, {} {:?}", from, to, seq1.len(), seq1),
                format!("{:?}->{:?}, {} {:?}", from, to, seq2.len(), seq2),
                format!("{:?}->{:?}, {} {:?}", from, to, seq3.len(), seq3),
                format!("{:?}->{:?}, {} {:?}", from, to, seq4.len(), seq4)
            ]
        }).flatten().collect(),
        grid: vec![],
    });
    DayOutput {
        silver_output: format!("{}", silver),
        gold_output: format!("{}", 0),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
    }
}

pub fn reverse_engineer_list(grid: &Grid<Action>, sequences: &Vec<Action>, errors: &mut Vec<String>) -> Vec<Vec<Action>> {
    if let Some((first, rest)) = sequences.split_first() {
        if let Some(start) = grid.find(|item| *item == *first) {
            reverse_engineer(&grid, Action::Empty, start, &vec![rest.to_vec()],  errors)
        } else {
            Vec::new()
        }
    } else {
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
    }).collect::<Vec<_>>();
    // errors.push(format!("Pruning from {} to {}", possibilities_before, all_output_sequences.len()));
    all_output_sequences
}

pub fn expand_numpad(sequence: &Vec<Action>, errors: &mut Vec<String>) -> Vec<Action> {
    let mut output = Vec::new();
    let mut last: Option<Action> = None;
    for item in sequence {
        if let Some(last_value) = last {
            match get_transition(last_value, *item) {
                Some(mut addition) => {
                    output.append(&mut addition);
                }
                None => {
                    return Vec::new();
                }
            }
        }
        last = Some(*item);
    }
    output
}

pub fn get_transition(previous: Action, next: Action) -> Option<Vec<Action>> {
    match (previous, next) {
        (Action::A, Action::A) => Some(vec![Action::A]),
        (Action::A, Action::Up) => Some(vec![Action::Left, Action::A]),
        (Action::A, Action::Down) => Some(vec![Action::Left, Action::Down, Action::A]), // !
        (Action::A, Action::Left) => Some(vec![Action::Down, Action::Left, Action::Left, Action::A]),
        (Action::A, Action::Right) => Some(vec![Action::Down, Action::A]),
        (Action::A, Action::Empty) => None,
        (Action::Up, Action::A) => Some(vec![Action::Right, Action::A]),
        (Action::Up, Action::Up) => Some(vec![Action::A]),
        (Action::Up, Action::Down) => Some(vec![Action::Down, Action::A]),
        (Action::Up, Action::Left) => Some(vec![Action::Down, Action::Left, Action::A]),
        (Action::Up, Action::Right) => Some(vec![Action::Down, Action::Right, Action::A]), // !
        (Action::Up, Action::Empty) => None,
        (Action::Down, Action::A) => Some(vec![Action::Up, Action::Right, Action::A]), // !!
        (Action::Down, Action::Up) => Some(vec![Action::Up, Action::A]),
        (Action::Down, Action::Down) => Some(vec![Action::A]),
        (Action::Down, Action::Left) => Some(vec![Action::Left, Action::A]),
        (Action::Down, Action::Right) => Some(vec![Action::Right, Action::A]),
        (Action::Down, Action::Empty) => None,
        (Action::Left, Action::A) => Some(vec![Action::Right, Action::Right, Action::Up, Action::A]),
        (Action::Left, Action::Up) => Some(vec![Action::Right, Action::Up, Action::A]),
        (Action::Left, Action::Down) => Some(vec![Action::Right, Action::A]),
        (Action::Left, Action::Left) => Some(vec![Action::A]),
        (Action::Left, Action::Right) => Some(vec![Action::Right, Action::Right, Action::A]),
        (Action::Left, Action::Empty) => None,
        (Action::Right, Action::A) => Some(vec![Action::Up, Action::A]),
        (Action::Right, Action::Up) => Some(vec![Action::Left, Action::Up, Action::A]), // !
        (Action::Right, Action::Down) => Some(vec![Action::Left, Action::A]),
        (Action::Right, Action::Left) => Some(vec![Action::Left, Action::Left, Action::A]),
        (Action::Right, Action::Right) => Some(vec![Action::A]),
        (Action::Right, Action::Empty) => None,
        (Action::Empty, Action::A) => None,
        (Action::Empty, Action::Up) => None,
        (Action::Empty, Action::Down) => None,
        (Action::Empty, Action::Left) => None,
        (Action::Empty, Action::Right) => None,
        (Action::Empty, Action::Empty) => None,
    }
}

pub fn get_transition_mul(previous: Action, next: Action) -> Vec<Vec<Action>> {
    match (previous, next) {
        (Action::A, Action::A) => vec!(vec![Action::A]),
        (Action::A, Action::Up) => vec!(vec![Action::Left, Action::A]),
        (Action::A, Action::Down) => vec!(
            vec![Action::Left, Action::Down, Action::A],
            vec![Action::Down, Action::Left, Action::A],
        ), // !
        (Action::A, Action::Left) => vec!(vec![Action::Down, Action::Left, Action::Left, Action::A]),
        (Action::A, Action::Right) => vec!(vec![Action::Down, Action::A]),
        (Action::A, Action::Empty) => Vec::new(),
        (Action::Up, Action::A) => vec!(vec![Action::Right, Action::A]),
        (Action::Up, Action::Up) => vec!(vec![Action::A]),
        (Action::Up, Action::Down) => vec!(vec![Action::Down, Action::A]),
        (Action::Up, Action::Left) => vec!(vec![Action::Down, Action::Left, Action::A]),
        (Action::Up, Action::Right) => vec!(
            vec![Action::Down, Action::Right, Action::A],
            vec![Action::Right, Action::Down, Action::A],
        ), // !
        (Action::Up, Action::Empty) => Vec::new(),
        (Action::Down, Action::A) => vec!(
            vec![Action::Up, Action::Right, Action::A],
            vec![Action::Right, Action::Up, Action::A],
        ), // !!
        (Action::Down, Action::Up) => vec!(vec![Action::Up, Action::A]),
        (Action::Down, Action::Down) => vec!(vec![Action::A]),
        (Action::Down, Action::Left) => vec!(vec![Action::Left, Action::A]),
        (Action::Down, Action::Right) => vec!(vec![Action::Right, Action::A]),
        (Action::Down, Action::Empty) => Vec::new(),
        (Action::Left, Action::A) => vec!(vec![Action::Right, Action::Right, Action::Up, Action::A]),
        (Action::Left, Action::Up) => vec!(vec![Action::Right, Action::Up, Action::A]),
        (Action::Left, Action::Down) => vec!(vec![Action::Right, Action::A]),
        (Action::Left, Action::Left) => vec!(vec![Action::A]),
        (Action::Left, Action::Right) => vec!(vec![Action::Right, Action::Right, Action::A]),
        (Action::Left, Action::Empty) => Vec::new(),
        (Action::Right, Action::A) => vec!(vec![Action::Up, Action::A]),
        (Action::Right, Action::Up) => vec!(
            vec![Action::Left, Action::Up, Action::A],
            vec![Action::Up, Action::Left, Action::A],
        ), // !
        (Action::Right, Action::Down) => vec!(vec![Action::Left, Action::A]),
        (Action::Right, Action::Left) => vec!(vec![Action::Left, Action::Left, Action::A]),
        (Action::Right, Action::Right) => vec!(vec![Action::A]),
        (Action::Right, Action::Empty) => Vec::new(),
        (Action::Empty, Action::A) => Vec::new(),
        (Action::Empty, Action::Up) => Vec::new(),
        (Action::Empty, Action::Down) => Vec::new(),
        (Action::Empty, Action::Left) => Vec::new(),
        (Action::Empty, Action::Right) => Vec::new(),
        (Action::Empty, Action::Empty) => Vec::new(),
    }
}

fn add_possibilities(sequences: Vec<Vec<Action>>, possibilities: Vec<Vec<Action>>, errors: &mut Vec<String>) -> Vec<Vec<Action>> {
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
            Action::Empty,
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

// <v<A>>^AvA^A<vA <AA>>^AAvA<^A >AAvA^A<vA >^AA<A>A<v<A>A >^AAAvA<^A>A
// <A>Av <<AA>^ AA>Av AA^A<v AAA>^A
// ^A<<^^A>>AvvvA
// 379A

// v<<A>>^AvA^Av <<A>>^AAv<A<A >>^AAvAA^<A>Av <A>^AA<A>Av<A< A>>^AAAvA^<A>A
// <A>A <AAv< AA>>^A vAA^Av <AAA>^A
// ^A ^^<<A >>AvvvA
// 379A


// <v<A>>^AvA^A<vA <AA>>^AAvA<^A >AAvA^A
// <A>Av <<AA >^AA >A
// ^A <<^^A
// 37

// v<<A>>^AvA^Av<<A >>^AAv<A<A >>^AAvAA^<A>Av<A
// <A>A< AA v<AA >>^A
// ^A ^^<<A
// 37

// <v<A>>^AvA^A<vA <A
// <A>Av <<AA >^AA >A
// ^A <<^^A
// v<<A>>^AvA^Av<<A >>^A
// <A>A< AA v<AA >>^A
// ^A ^^<<A

// <vA<AA>>^AAvA<^A>AAvA^A<vA>^
// v<<AA>^AA>Av
// <<^^A
// 3->7
// v<<A>>^AAv<A<A>>^AAvAA^<A>Av<A>^
// <AAv<AA>>^Av
// ^^<<A
// 3->7

// [Down, Left, Left, A, Right, Right, Up, A, Down, A, Up, A, Down,
// Left, Left, A, Right, Right, Up, A, A, Down, Left, A, Left, A,
// Right, Right, Up, A, A, Down, A, A, Up, Left, A, Right, A, Down,
// Left, A, Right, Up, A, A, Left, A, Right, A, Down, Left, A, Left,
// A, Right, Right, Up, A, A, A, Down, A, Up, Left, A, Right, A]

