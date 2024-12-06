use std::collections::HashSet;
use crate::app::{DayOutput, Diagnostic, GridCell, Tab};

#[derive(Clone, Debug)]
pub enum Letter {
    Dot,
    Hash,
    Guard,
}

pub type Coord = (i32, i32);

pub fn puzzle(input: &str) -> DayOutput {
    let mut errors: Vec<String> = Vec::new();
    let mut starting_position = None;
    let mut tabs = Vec::new();
    let mut grid = input.split("\n").enumerate().map(|(y, line)| {
        log::info!("{:?}", line);
        line.chars().enumerate().filter_map(|(x, character)| {
            log::info!("{:?}", character);
            match character {
                '.' => Some(Letter::Dot),
                '#' => Some(Letter::Hash),
                '^' => {
                    starting_position = Some((x as i32, y as i32));
                    Some(Letter::Guard)
                }
                c => {
                    errors.push(format!("Found invalid character in input: {}", c));
                    None
                }
            }
        }).collect::<Vec<Letter>>()
    }).collect::<Vec<Vec<Letter>>>();

    let (ordinary_visited, blockage_locations) = if let Some(starting_position) = starting_position {
        let starting_dir = (0, -1);
        let (_looped, ordinary_visited_set) = find_visited(&grid, starting_position, starting_dir);
        let mut ordinary_visited = ordinary_visited_set.into_iter().collect::<Vec<_>>();
        ordinary_visited.sort();

        let mut blockage_locations = HashSet::new();
        for (blockage_index, blockage_location) in ordinary_visited.iter().enumerate() {
            let blockage_location = blockage_location.clone();
            if let Some(handle) = find_coord_mut(&mut grid, blockage_location) {
                let original = handle.clone();
                *handle = Letter::Hash;

                let (looped, visited) = find_visited(&grid, starting_position, starting_dir);

                if looped {
                    blockage_locations.insert(blockage_location);
                    if tabs.len() < 100 {
                        add_tab(&mut tabs, &grid, &visited, format!("B{}", blockage_index));
                    }
                }

                if let Some(handle) = find_coord_mut(&mut grid, blockage_location) {
                    *handle = original;
                } else {
                    errors.push(format!("Couldn't reset coord {:?}", blockage_location));
                }
            }
        }
        (ordinary_visited, blockage_locations)
    } else {
        (Vec::new(), HashSet::new())
    };


    DayOutput {
        silver_output: format!("{}", ordinary_visited.len()),
        gold_output: format!("{}", blockage_locations.len()),
        diagnostic: Diagnostic::with_tabs(tabs, format!("errors: {:?}, starting_position: {:?}", errors, starting_position)),
    }
}

fn add_tab(tabs: &mut Vec<Tab>, grid: &Vec<Vec<Letter>>, visited: &HashSet<Coord>, title: String) {
    let grid = grid.iter().enumerate().map(|(y, row)|{
        row.iter().enumerate().map(|(x, letter)|{
            let text = match letter {
                Letter::Dot => {
                    if visited.contains(&(x as i32, y as i32)) {
                        "X"
                    } else {
                        "."
                    }
                },
                Letter::Hash => "#",
                Letter::Guard => "^",
            };
            GridCell {
                text: text.to_string(),
                class: Default::default(),
            }
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();
    let tab = Tab {
        title,
        strings: vec![],
        grid,
    };
    tabs.push(tab);
}

fn find_visited(grid: &Vec<Vec<Letter>>, starting_position: Coord, starting_dir: Coord) -> (bool, HashSet<Coord>) {
    let mut visited_locations = HashSet::new();
    let mut visited_location_directions = HashSet::new();
    let mut current_position = starting_position.clone();
    let mut current_dir = starting_dir.clone();
    let looped = loop {
        let location_direction = (current_position, current_dir);
        if visited_location_directions.contains(&location_direction) {
            break true;
        }
        visited_location_directions.insert(location_direction);
        visited_locations.insert(current_position);
        let next_position = add_coord(current_position, current_dir);
        let next_tile = find_coord(&grid, next_position);
        match next_tile {
            None => {
                // We walked outside of bounds, we're done
                break false;
            }
            Some(tile_type) => {
                match tile_type {
                    Letter::Guard |
                    Letter::Dot => {
                        current_position = next_position;
                    }
                    Letter::Hash => {
                        current_dir = (-current_dir.1, current_dir.0);
                    }
                }
            }
        }
    };
    (looped, visited_locations)
}

fn find_coord(grid: &Vec<Vec<Letter>>, coord: Coord) -> Option<Letter> {
    let (x, y): (Option<usize>, Option<usize>) = (coord.0.try_into().ok(), coord.1.try_into().ok());
    if let (Some(x), Some(y)) = (x, y) {
        grid.get(y).map(|row| {
            row.get(x).map(|letter| letter.clone())
        }).unwrap_or(None)
    } else {
        None
    }
}

fn add_coord(first: Coord, second: Coord) -> Coord {
    (first.0 + second.0, first.1 + second.1)
}

fn find_coord_mut(grid: &mut Vec<Vec<Letter>>, coord: Coord) -> Option<&mut Letter> {
    let (x, y): (Option<usize>, Option<usize>) = (coord.0.try_into().ok(), coord.1.try_into().ok());
    if let (Some(x), Some(y)) = (x, y) {
        grid.get_mut(y).map(|row| {
            row.get_mut(x)
        }).unwrap_or(None)
    } else {
        None
    }
}
