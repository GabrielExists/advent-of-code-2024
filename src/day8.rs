use std::collections::HashMap;
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::grid::{Coord, Grid};

const ANTIPOLE_CHAR: char = '#';

pub fn puzzle(input: &str) -> DayOutput {
    let mut antenna_lists: HashMap<char, Vec<Coord>> = HashMap::new();
    let input_grid = Grid::from_with_index(input, |character, x, y| {
        match character {
            '.' => {}
            antenna => {
                let entry = antenna_lists.entry(antenna).or_insert(Vec::new());
                entry.push(Coord::new(x as i32, y as i32));
            }
        }
        character
    });

    let mut grid = input_grid.clone();
    let mut grid_repeating = input_grid.clone();
    for (_, antenna_list) in antenna_lists.iter() {
        for (index, antenna_one) in antenna_list.iter().enumerate() {
            for antenna_two in antenna_list[index + 1..].iter() {
                add_antipole(&mut grid, antenna_one, antenna_two);
                add_antipole(&mut grid, antenna_two, antenna_one);
                add_antipole_repeating(&mut grid_repeating, antenna_one, antenna_two);
                add_antipole_repeating(&mut grid_repeating, antenna_two, antenna_one);
            }
        }
    }

    let predicate = |character: &char| { *character == '#' };
    let sum = grid.count(predicate);
    let sum_repeating = grid_repeating.count(predicate);
    let tabs = vec![
        Tab {
            title: "Input".to_string(),
            strings: vec![],
            grid: input_grid.to_tab_grid(),
        },
        Tab {
            title: "Output".to_string(),
            strings: vec![],
            grid: grid.to_tab_grid(),
        },
        Tab {
            title: "Output repeating".to_string(),
            strings: vec![],
            grid: grid_repeating.to_tab_grid(),
        },
    ];
    DayOutput {
        silver_output: format!("{}", sum),
        gold_output: format!("{}", sum_repeating),
        diagnostic: Diagnostic::with_tabs(tabs, format!("")),
    }
}

fn add_antipole(grid: &mut Grid<char>, antenna_one: &Coord, antenna_two: &Coord) {
    let difference = antenna_one.subtract(antenna_two);
    let antipole = antenna_one.add(&difference);
    if let Some(cell) = grid.get_mut(antipole) {
        *cell = ANTIPOLE_CHAR;
    }
}

fn add_antipole_repeating(grid: &mut Grid<char>, antenna_one: &Coord, antenna_two: &Coord) {
    let difference = antenna_one.subtract(antenna_two);
    let mut scalar = 0;
    loop {
        let current_difference = difference.multiply(scalar);
        let antipole = antenna_one.add(&current_difference);
        if let Some(cell) = grid.get_mut(antipole) {
            *cell = ANTIPOLE_CHAR;
        } else {
            break;
        }
        scalar += 1;
    }
}

