use std::collections::{HashMap, HashSet};
use crate::app::{DayOutput, Diagnostic, GridCell, Tab};

type Coord = (i32, i32);

pub fn puzzle(input: &str) -> DayOutput {
    let mut errors: Vec<String> = Vec::new();
    let mut antenna_lists: HashMap<char, Vec<Coord>> = HashMap::new();
    let row = input.split("\n").collect::<Vec<&str>>();
    let num_rows = row.len();
    let mut num_columns = None;
    let mut grid = Vec::new();
    for (y, row) in row.into_iter().enumerate() {
        if let None = num_columns {
            num_columns = Some(row.len());
        }
        let mut grid_row = Vec::new();
        for (x, character) in row.chars().into_iter().enumerate() {
            match character {
                '.' => {}
                antenna => {
                    let entry = antenna_lists.entry(antenna).or_insert(Vec::new());
                    entry.push((x as i32, y as i32));
                }
            }
            grid_row.push(GridCell {
                text: character.to_string(),
                class: Default::default(),
            })
        }
        grid.push(grid_row);
    }

    let mut antipoles = HashSet::new();
    for (_, antenna_list) in antenna_lists.iter() {
        for (index, antenna_one) in antenna_list.iter().enumerate() {
            for antenna_two in antenna_list[index + 1..].iter() {
                find_antipole(&mut antipoles, antenna_one, antenna_two, num_rows, num_columns);
                find_antipole(&mut antipoles, antenna_two, antenna_one, num_rows, num_columns);
            }
        }
    }

    log::info!("{:?}", grid);
    let mut pole_grid = grid.clone();
    apply_antipoles_to_grid(&mut pole_grid, &antipoles);
    let sum = pole_grid.iter().fold(0, |acc, row|{
        let subsum = row.iter().fold(0, |acc, cell| {
            if cell.text == "#" {
                acc + 1
            } else {
                acc
            }
        });
        acc + subsum
    });
    let mut sorted_antipoles = antipoles.iter().collect::<Vec<_>>();
    sorted_antipoles.sort();
    let tabs = vec![
        Tab {
            title: "Input".to_string(),
            strings: vec![],
            grid,
        },
        Tab {
            title: "Output".to_string(),
            strings: vec![],
            grid: pole_grid,
        },
        Tab {
            title: format!("list of length {}", sorted_antipoles.len()),
            strings: sorted_antipoles.into_iter().map(|a|format!("({},{})", a.0, a.1)).collect::<Vec<_>>(),
            grid: vec![],
        },
    ];
    DayOutput {
        silver_output: format!("{}", antipoles.len()),
        gold_output: format!("{}", 0),
        diagnostic: Diagnostic::with_tabs(tabs, format!("sum of debug view {}, rows {:?}, cols {:?}", sum, num_rows, num_columns)),
    }
}

fn apply_antipoles_to_grid(grid: &mut Vec<Vec<GridCell>>, antipoles: &HashSet<Coord>) {
    for antipole in antipoles.iter() {
        if let Some(cell) = find_coord_mut(grid, *antipole) {
            *cell = GridCell {
                text: "#".to_string(),
                class: Default::default(),
            };
        }
    }
}

fn find_antipole(antipoles: &mut HashSet<Coord>, antenna_one: &Coord, antenna_two: &Coord, num_row: usize, num_column: Option<usize>) {
    let difference = subtract_coord(*antenna_one, *antenna_two);
    let antipole = add_coord(*antenna_one, difference);
    if in_bounds(num_row, num_column, antipole) {
        if !antipoles.contains(&antipole) {
            antipoles.insert(antipole);
        }
    }
}

fn in_bounds(num_row: usize, num_column: Option<usize>, coord: Coord) -> bool {
    if let Some(num_column) = num_column {
        let (x, y): (Option<usize>, Option<usize>) = (coord.0.try_into().ok(), coord.1.try_into().ok());
        if let (Some(x), Some(y)) = (x, y) {
            x < num_column && y < num_row && x >= 0 && y >= 0
        } else {
            false
        }
    } else {
        false
    }
}

fn add_coord(first: Coord, second: Coord) -> Coord {
    (first.0 + second.0, first.1 + second.1)
}

fn subtract_coord(first: Coord, second: Coord) -> Coord {
    (first.0 - second.0, first.1 - second.1)
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
