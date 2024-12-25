use std::collections::HashMap;
use std::hash::Hash;
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::grid::{Coord, Grid};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum Action {
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
    let errors: Vec<String> = Vec::new();
    let inputs = input.split("\n").into_iter().filter_map(|line| {
        let code = line.trim_matches('A').parse::<u64>().ok();
        let line = line.chars().filter_map(|char|{
            match char {
                'A' => Some(NumpadKey::A),
                char => {
                    char.to_digit(10).map(|dig|NumpadKey::Number(dig as u8))
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
        let actions_one = reverse_engineer(&numpad_grid, numpad_start, &input_row);
        stepped_outputs.push(format!("{:?}", actions_one));
        let actions_two = reverse_engineer(&direction_grid, direction_start, &actions_one);
        stepped_outputs.push(format!("{:?}", actions_two));
        let actions_three = reverse_engineer(&direction_grid, direction_start, &actions_two);
        stepped_outputs.push(format!("{:?}", actions_three));
        let length = actions_three.len();
        silver += code * length as u64;
        outputs.push((actions_three, code, length));
    }
    tabs.push(Tab {
        title: "Outputs".to_string(),
        strings: outputs.iter().map(|(actions, code, length)| format!("Length {}, Code {}", length, code)).collect(),
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

pub fn reverse_engineer<T>(grid: &Grid<T>, start_pos: Coord, sequence: &Vec<T>) -> Vec<Action>
    where T: Eq + Hash + Clone {
    let mut coordinate_lookup: HashMap<T, Coord> = HashMap::new();
    let _ = grid.map_grid(|cell, x, y| {
        coordinate_lookup.entry(cell.clone()).or_insert(Coord((x as i32, y as i32)));
    });
    let mut output_sequence = Vec::new();
    let mut current_pos = start_pos;
    for item in sequence.iter() {
        if let Some(new_pos) = coordinate_lookup.get(item) {
            let delta = new_pos.subtract(&current_pos);
            let Coord((delta_x, delta_y)) = delta;
            if delta_x > 0 {
                for _ in 0..delta_x {
                    output_sequence.push(Action::Right);
                }
            }
            if delta_y > 0 {
                for _ in 0..delta_y {
                    output_sequence.push(Action::Down);
                }
            }
            if delta_y < 0 {
                for _ in 0..-delta_y {
                    output_sequence.push(Action::Up);
                }
            }
            if delta_x < 0 {
                for _ in 0..-delta_x {
                    output_sequence.push(Action::Left);
                }
            }
            output_sequence.push(Action::A);
            current_pos = *new_pos;
        }
    }
    output_sequence
}

fn add_possibilities(sequences: Vec<Vec<Action>>, possibilities: Vec<Vec<Action>>) -> Vec<Vec<Action>> {
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