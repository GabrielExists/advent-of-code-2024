use std::collections::{HashMap, HashSet};
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::day6_display::*;

pub type Coord = (i32, i32);
pub type Grid = Vec<Vec<Letter>>;

#[derive(Clone, Debug)]
pub enum Letter {
    Dot,
    Hash,
    Guard,
}

// Location, Direction
pub type Footstep = (Coord, Coord);

pub fn puzzle(input: &str) -> DayOutput {
    let mut errors: Vec<String> = Vec::new();
    let mut starting_position = None;
    let grid = input.split("\n").enumerate().map(|(y, line)| {
        line.chars().enumerate().filter_map(|(x, character)| {
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
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    let mut tabs = Vec::new();
    let mut visited_locations: HashSet<Footstep> = HashSet::new();
    let mut blockage_locations = HashSet::new();
    if let Some(starting_position) = starting_position {
        let mut current_position = starting_position.clone();
        let mut facing_dir = (0, -1);
        let mut step_number = 0;
        loop {
            visited_locations.insert((current_position, facing_dir));
            let next_position = add_coord(current_position, facing_dir);
            let next_tile = find_coord(&grid, next_position);
            match next_tile {
                None => {
                    // We walked outside of bounds, we're done
                    break;
                }
                Some(tile_type) => {
                    let dir_to_the_right = (-facing_dir.1, facing_dir.0);
                    match tile_type {
                        Letter::Guard |
                        Letter::Dot => {
                            let (loopable, mut output_grid) = check_for_loopability(&grid, &visited_locations, next_position, current_position, facing_dir);
                            apply_locations_to_output_grid(&mut output_grid, blockage_locations.iter().map(|a: &Coord| a.clone()), OutputLetter::Obstacle);
                            apply_locations_to_output_grid(&mut output_grid, vec![next_position].into_iter(), OutputLetter::CheckingObstacle);

                            if loopable {
                                // If we'd walk right here, we'd end up looping, so we can add a blockage in front of here
                                blockage_locations.insert(next_position);
                                apply_locations_to_output_grid(&mut output_grid, vec![next_position].into_iter(), OutputLetter::Obstacle);
                            }
                            if tabs.len() < 100 && loopable {
                                tabs.push(Tab {
                                    title: if loopable {
                                        format!("O {}", step_number)
                                    } else {
                                        format!("No {}", step_number)
                                    },
                                    strings: vec![],
                                    grid: cells_from_output_grid(&output_grid),
                                });
                            }
                            step_number += 1;
                            current_position = next_position;
                        }
                        Letter::Hash => {
                            facing_dir = dir_to_the_right;
                        }
                    }
                    // Check for loopability
                }
            }
        }
    }

    let mut counting_grid = output_grid_from_grid(&grid);
    apply_locations_to_output_grid(&mut counting_grid, blockage_locations.iter().map(|a| (a).clone()), OutputLetter::Counter);
    let num_blockage_locations = counting_grid.into_iter().fold(0, |acc, row| {
        let row_count = row.into_iter().fold(0, |acc, cell| {
            match cell {
                OutputLetter::Counter => acc + 1,
                _ => acc,
            }
        });
        row_count + acc
    });

    let output_grid = output_grid_from_grid(&grid);
    let rows = cells_from_output_grid(&output_grid);
    tabs.insert(0, Tab {
        title: "Starting state".to_string(),
        strings: vec![],
        grid: rows,
    });

    let mut output_grid = output_grid_from_grid(&grid);
    apply_locations_to_output_grid(&mut output_grid, visited_locations.iter().map(|a| (*a).0.clone()), OutputLetter::Walked);
    apply_locations_to_output_grid(&mut output_grid, blockage_locations.iter().map(|a| a.clone()), OutputLetter::Obstacle);
    let rows = cells_from_output_grid(&output_grid);
    let tab = Tab {
        title: "Complete walk".to_string(),
        strings: vec![],
        grid: rows,
    };
    tabs.insert(1, tab);

    DayOutput {
        silver_output: format!("{}", visited_locations.len()),
        gold_output: format!("{}", num_blockage_locations),
        diagnostic: Diagnostic::with_tabs(tabs, format!("errors: {:?}, starting_position: {:?}", errors, starting_position)),
    }
}

fn check_for_loopability(grid: &Grid, visited_locations: &HashSet<Footstep>, obstacle: Coord, starting_position: Coord, starting_dir: Coord) -> (bool, Vec<Vec<OutputLetter>>) {
    let mut current_position = starting_position;
    let mut facing_dir = starting_dir;
    let mut visited_locations = visited_locations.clone();
    let loopable = loop {
        let footstep = (current_position, facing_dir);
        if visited_locations.contains(&footstep) {
            // We've walked here, in the same direction, before
            break true;
        }
        visited_locations.insert(footstep);
        let next_position = add_coord(current_position, facing_dir);
        let next_tile = find_coord(&grid, next_position);
        match next_tile {
            None => {
                // We walked outside of bounds, we're done
                break false;
            }
            Some(next_tile_type) => {
                let blocked = match next_tile_type {
                    Letter::Guard |
                    Letter::Dot => {
                        false
                    }
                    Letter::Hash => {
                        true
                    }
                };
                if blocked || next_position == obstacle {
                    let dir_to_the_right = (-facing_dir.1, facing_dir.0);
                    facing_dir = dir_to_the_right;
                } else {
                    current_position = next_position;
                }
                // Check for loopability
            }
        }
    };
    let mut output_grid = output_grid_from_grid(&grid);
    // apply_locations_to_output_grid(&mut output_grid, initial_visited_locations.keys().map(|a| a.clone()), OutputLetter::Walked);
    // apply_locations_to_output_grid(&mut output_grid, checked_locations.iter().map(|a| a.clone()), OutputLetter::Checked);
    // apply_locations_to_output_grid(&mut output_grid, vec![starting_position].into_iter(), OutputLetter::CheckingStartLocation);
    (loopable, output_grid)
}

pub fn find_coord(grid: &Vec<Vec<Letter>>, coord: (i32, i32)) -> Option<Letter> {
    let (x, y): (Option<usize>, Option<usize>) = (coord.0.try_into().ok(), coord.1.try_into().ok());
    if let (Some(x), Some(y)) = (x, y) {
        grid.get(y).map(|row| {
            row.get(x).map(|letter| letter.clone())
        }).unwrap_or(None)
    } else {
        None
    }
}

pub fn add_coord(first: (i32, i32), second: (i32, i32)) -> (i32, i32) {
    (first.0 + second.0, first.1 + second.1)
}
