use std::collections::HashMap;
use std::hash::Hash;
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::grid::{Coord, Grid};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    A,
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
        let actions_one = reverse_engineer(&numpad_grid, NumpadKey::Empty, numpad_start, &vec![input_row], &mut errors);
        let actions_two = reverse_engineer(&direction_grid, Action::Empty, direction_start, &actions_one, &mut errors);
        let actions_three = reverse_engineer(&direction_grid, Action::Empty, direction_start, &actions_two, &mut errors);
        // stepped_outputs.push(format!("{:?}", actions_one));
        // stepped_outputs.push(format!("{:?}", actions_two));
        let shortest = actions_three.into_iter().min_by_key(|sequence| sequence.len()).unwrap_or(Vec::new());
        stepped_outputs.push(format!("{:?}", shortest));
        let length = shortest.len();
        silver += code * length as u64;
        outputs.push((shortest, code, length));
    }
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
    DayOutput {
        silver_output: format!("{}", silver),
        gold_output: format!("{}", 0),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
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
    all_output_sequences.into_iter().filter(|sequence| {
        playback_ok(grid, blank.clone(), start_pos, sequence)
    }).collect()
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

