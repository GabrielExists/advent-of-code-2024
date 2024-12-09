use std::collections::HashSet;
use std::ops::Deref;
use crate::app::{DayOutput, Diagnostic, GridCell, Tab};
use crate::grid::{Coord, Grid};

#[derive(Clone, Debug)]
pub enum Letter {
    Dot,
    Hash,
    Guard,
}

pub fn puzzle(input: &str) -> DayOutput {
    let mut errors: Vec<String> = Vec::new();
    let mut starting_position = None;
    let mut tabs = Vec::new();
    let mut grid = Grid::from_with_index_filtered(input, |character, x, y| {
        match character {
            '.' => Some(Letter::Dot),
            '#' => Some(Letter::Hash),
            '^' => {
                starting_position = Some(Coord::new(x as i32, y as i32));
                Some(Letter::Guard)
            }
            c => {
                errors.push(format!("Found invalid character in input: {}", c));
                None
            }
        }
    });

    let (ordinary_visited, blockage_locations) = if let Some(starting_position) = starting_position {
        let starting_dir = Coord::new(0, -1);
        let (_looped, ordinary_visited_set) = find_visited(&grid, starting_position, starting_dir);
        let mut ordinary_visited = ordinary_visited_set.into_iter().collect::<Vec<_>>();
        ordinary_visited.sort();

        let mut blockage_locations: HashSet<Coord> = HashSet::new();
        for (blockage_index, blockage_location) in ordinary_visited.iter().enumerate() {
            let blockage_location: Coord = blockage_location.clone();
            if let Some(handle) = grid.get_mut(blockage_location) {
                let original = handle.clone();
                *handle = Letter::Hash;

                let (looped, visited) = find_visited(&grid, starting_position, starting_dir);

                if looped {
                    blockage_locations.insert(blockage_location);
                    if tabs.len() < 100 {
                        add_tab(&mut tabs, &grid, &visited, format!("B{}", blockage_index));
                    }
                }

                if let Some(handle) = grid.get_mut(blockage_location) {
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

fn add_tab(tabs: &mut Vec<Tab>, grid: &Grid<Letter>, visited: &HashSet<Coord>, title: String) {
    let grid = grid.map_grid(|letter, x, y| {
        let text = match letter {
            Letter::Dot => {
                let coord = Coord::new(x as i32, y as i32);
                if visited.contains(&coord) {
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
    });
    let tab = Tab {
        title,
        strings: vec![],
        grid,
    };
    tabs.push(tab);
}

fn find_visited(grid: &Grid<Letter>, starting_position: Coord, starting_dir: Coord) -> (bool, HashSet<Coord>) {
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
        let next_position = current_position.add(&current_dir);
        let next_tile = grid.get(next_position);
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
                        current_dir = Coord::new(-current_dir.deref().1, current_dir.deref().0);
                    }
                }
            }
        }
    };
    (looped, visited_locations)
}

