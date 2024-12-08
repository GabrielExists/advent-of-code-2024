use std::collections::HashMap;
use crate::app::{DayOutput, Diagnostic, GridCell, Tab};

type Coord = (i32, i32);
type Grid = Vec<Vec<char>>;

const ANTIPOLE_CHAR: char = '#';

pub fn puzzle(input: &str) -> DayOutput {
    let mut antenna_lists: HashMap<char, Vec<Coord>> = HashMap::new();
    let input_grid = input.split("\n").enumerate().map(|(y, row)| {
        row.chars().enumerate().map(|(x, character)| {
            match character {
                '.' => {}
                antenna => {
                    let entry = antenna_lists.entry(antenna).or_insert(Vec::new());
                    entry.push((x as i32, y as i32));
                }
            }
            character
        }).collect::<Vec<_>>()
    }).filter(|a| !a.is_empty()).collect::<Vec<Vec<char>>>();

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

    let sum = count_antipoles(&grid);
    let sum_repeating = count_antipoles(&grid_repeating);
    let tabs = vec![
        Tab {
            title: "Input".to_string(),
            strings: vec![],
            grid: tab_grid_from_grid(&input_grid),
        },
        Tab {
            title: "Output".to_string(),
            strings: vec![],
            grid: tab_grid_from_grid(&grid),
        },
        Tab {
            title: "Output repeating".to_string(),
            strings: vec![],
            grid: tab_grid_from_grid(&grid_repeating),
        },
    ];
    DayOutput {
        silver_output: format!("{}", sum),
        gold_output: format!("{}", sum_repeating),
        diagnostic: Diagnostic::with_tabs(tabs, format!("")),
    }
}

fn count_antipoles(pole_grid: &Vec<Vec<char>>) -> i32 {
    let sum = pole_grid.iter().fold(0, |acc, row| {
        let subsum = row.iter().fold(0, |acc, cell| {
            if *cell == '#' {
                acc + 1
            } else {
                acc
            }
        });
        acc + subsum
    });
    sum
}

fn add_antipole(grid: &mut Grid, antenna_one: &Coord, antenna_two: &Coord) {
    let difference = subtract_coord(*antenna_one, *antenna_two);
    let antipole = add_coord(*antenna_one, difference);
    if let Some(cell) = find_coord_mut(grid, antipole) {
        *cell = ANTIPOLE_CHAR;
    }
}

fn add_antipole_repeating(grid: &mut Grid, antenna_one: &Coord, antenna_two: &Coord) {
    let difference = subtract_coord(*antenna_one, *antenna_two);
    let mut scalar = 0;
    loop {
        let current_difference = multiply_coord(difference, scalar);
        let antipole = add_coord(*antenna_one, current_difference);
        if let Some(cell) = find_coord_mut(grid, antipole) {
            *cell = ANTIPOLE_CHAR;
        } else {
            break;
        }
        scalar += 1;
    }
}

fn add_coord(first: Coord, second: Coord) -> Coord {
    (first.0 + second.0, first.1 + second.1)
}

fn subtract_coord(first: Coord, second: Coord) -> Coord {
    (first.0 - second.0, first.1 - second.1)
}

fn multiply_coord(coord: Coord, scalar: i32) -> Coord {
    (coord.0 * scalar, coord.1 * scalar)
}

fn find_coord_mut<T>(grid: &mut Vec<Vec<T>>, coord: Coord) -> Option<&mut T> {
    let (x, y): (Option<usize>, Option<usize>) = (coord.0.try_into().ok(), coord.1.try_into().ok());
    if let (Some(x), Some(y)) = (x, y) {
        grid.get_mut(y).map(|row| {
            row.get_mut(x)
        }).unwrap_or(None)
    } else {
        None
    }
}

fn tab_grid_from_grid(grid: &Grid) -> Vec<Vec<GridCell>> {
    grid.into_iter().map(|row| {
        row.into_iter().map(|cell| {
            GridCell {
                text: format!("{}", cell),
                class: Default::default(),
            }
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>()
}