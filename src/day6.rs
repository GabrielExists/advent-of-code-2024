use std::collections::HashSet;
use crate::app::DayOutput;

#[derive(Clone, Debug)]
enum Letter {
    Dot,
    Hash,
    Guard,
}

pub fn puzzle(input: &str) -> DayOutput {
    let mut errors: Vec<String> = Vec::new();
    let mut starting_position = None;
    let grid = input.split("\n").enumerate().map(|(y, line)| {
        log::info!("{:?}", line);
        line.chars().enumerate().filter_map(|(x, character)| {
            log::info!("{:?}", character);
            match character {
                '.' => Some(Letter::Dot),
                '#' => Some(Letter::Hash),
                '^' => {
                    starting_position = Some((x as i32, y as i32));
                    Some(Letter::Guard)
                },
                c => {
                    errors.push(format!("Found invalid character in input: {}", c));
                    None
                }
            }
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    let mut visited_locations = HashSet::new();
    let mut history = Vec::new();
    if let Some(starting_position) = starting_position {
        let mut current_position = starting_position.clone();
        let mut facing_dir = (0, -1);
        loop {
            visited_locations.insert(current_position);
            let next_position = add_coord(current_position, facing_dir);
            let next_tile = find_coord(&grid, next_position);
            match next_tile {
                None => {
                    history.push(format!("Break with current: {:?}, next: {:?}", current_position, next_position));
                    // We walked outside of bounds, we're done
                    break;
                }
                Some(tile_type) => {
                    match tile_type {
                        Letter::Guard |
                        Letter::Dot => {
                            history.push(format!("Walk {:?} with current: {:?}, next: {:?}", facing_dir, current_position, next_position));
                            current_position = next_position;
                        }
                        Letter::Hash => {
                            history.push(format!("Turn {:?} with current: {:?}, next: {:?}", facing_dir, current_position, next_position));
                            facing_dir = (-facing_dir.1, facing_dir.0);
                        }
                    }
                }
            }
        }

    }


    DayOutput {
        silver_output: format!("{}", visited_locations.len()),
        gold_output: format!("{}", 0),
        diagnostic: format!("errors: {:?}, history: {:?}, starting_position: {:?}", errors, history, starting_position),
    }
}

fn find_coord(grid: &Vec<Vec<Letter>>, coord: (i32, i32)) -> Option<Letter> {
    let (x, y): (Option<usize>, Option<usize>) = (coord.0.try_into().ok(), coord.1.try_into().ok());
    if let (Some(x), Some(y)) = (x, y) {
        grid.get(y).map(|row| {
            row.get(x).map(|letter| letter.clone())
        }).unwrap_or(None)
    } else {
        None
    }
}

fn add_coord(first: (i32, i32), second: (i32, i32)) -> (i32, i32) {
    (first.0 + second.0, first.1 + second.1)
}