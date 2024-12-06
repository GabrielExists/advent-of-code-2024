use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter, Write};
use std::process::Output;
use crate::app::{class_string, DayOutput, Diagnostic, GridCell, Tab};

const ENABLE_HISTORY: bool = false;

#[derive(Clone, Debug)]
enum Letter {
    Dot,
    Hash,
    Guard,
}

#[derive(Clone, Debug)]
enum OutputLetter {
    Dot,
    Hash,
    Guard,
    Walked,
    Checked,
    Obstacle,
    CheckingStartLocation,
    CheckingObstacle,
    Error,
}

impl Display for OutputLetter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputLetter::Dot => f.write_str("."),
            OutputLetter::Hash => f.write_str("#"),
            OutputLetter::Guard => f.write_str("^"),
            OutputLetter::Walked => f.write_str("X"),
            OutputLetter::Checked => f.write_str("/"),
            OutputLetter::Obstacle => f.write_str("O"),
            OutputLetter::CheckingStartLocation => f.write_str("C"),
            OutputLetter::CheckingObstacle => f.write_str("Ø"),
            OutputLetter::Error => f.write_str("!"),
        }
    }
}

impl From<&Letter> for OutputLetter {
    fn from(value: &Letter) -> Self {
        match value {
            Letter::Dot => Self::Dot,
            Letter::Hash => Self::Hash,
            Letter::Guard => Self::Guard,
        }
    }
}

impl OutputLetter {
    fn to_cell(&self) -> GridCell {
        let class = match self {
            OutputLetter::Dot => "",
            OutputLetter::Hash => "",
            _ => "",
        };
        GridCell {
            text: self.to_string(),
            class: class_string(&class),
        }
    }
}

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
    let mut visited_locations: HashMap<Coord, HashSet<Coord>> = HashMap::new();
    let mut blockage_locations = HashSet::new();
    let mut history = Vec::new();
    if let Some(starting_position) = starting_position {
        let mut current_position = starting_position.clone();
        let mut facing_dir = (0, -1);
        let mut step_number = 0;
        loop {
            let footsteps_here = visited_locations.entry(current_position).or_insert(HashSet::new());
            footsteps_here.insert(facing_dir);
            let next_position = add_coord(current_position, facing_dir);
            let next_tile = find_coord(&grid, next_position);
            match next_tile {
                None => {
                    if ENABLE_HISTORY {
                        history.push(format!("Break with current: {:?}, next: {:?}", current_position, next_position));
                    }
                    // We walked outside of bounds, we're done
                    break;
                }
                Some(tile_type) => {
                    let dir_to_the_right = (-facing_dir.1, facing_dir.0);
                    match tile_type {
                        Letter::Guard |
                        Letter::Dot => {
                            let (loopable, mut output_grid) = check_for_loopability(&grid, &visited_locations, current_position, dir_to_the_right);
                            apply_locations_to_output_grid(&mut output_grid, blockage_locations.iter().map(|a: &Coord| a.clone()), OutputLetter::Obstacle);
                            apply_locations_to_output_grid(&mut output_grid, vec![next_position].into_iter(), OutputLetter::CheckingObstacle);
                            if loopable {
                                // If we'd walk right here, we'd end up looping, so we can add a blockage in front of here
                                if ENABLE_HISTORY {
                                    history.push(format!("Add blockage at {:?}", next_position));
                                }
                                blockage_locations.insert(next_position);
                                apply_locations_to_output_grid(&mut output_grid, vec![next_position].into_iter(), OutputLetter::Obstacle);
                                if step_number < 30 {
                                    tabs.push(Tab {
                                        title: format!("Ob {}", step_number),
                                        strings: vec![],
                                        grid: cells_from_output_grid(&output_grid)
                                    });
                                    step_number += 1;
                                }
                            } else {
                                if step_number < 30 {
                                    tabs.push(Tab {
                                        title: format!("No {}", step_number),
                                        strings: vec![],
                                        grid: cells_from_output_grid(&output_grid),
                                    });
                                    step_number += 1;
                                }
                            }
                            if ENABLE_HISTORY {
                                history.push(format!("Walk {:?} with current: {:?}, next: {:?}", facing_dir, current_position, next_position));
                            }
                            current_position = next_position;
                        }
                        Letter::Hash => {
                            if ENABLE_HISTORY {
                                history.push(format!("Turn {:?} with current: {:?}, next: {:?}", facing_dir, current_position, next_position));
                            }
                            facing_dir = dir_to_the_right;
                        }
                    }
                    // Check for loopability
                }
            }
        }
    }

    let mut output_grid = output_grid_from_grid(&grid);
    let rows = cells_from_output_grid(&output_grid);
    tabs.insert(0, Tab {
        title: "Starting state".to_string(),
        strings: vec![],
        grid: rows,
    });

    let mut output_grid = output_grid_from_grid(&grid);
    apply_locations_to_output_grid(&mut output_grid, visited_locations.keys().map(|a|a.clone()), OutputLetter::Walked);
    apply_locations_to_output_grid(&mut output_grid, blockage_locations.iter().map(|a|a.clone()), OutputLetter::Obstacle);
    let rows = cells_from_output_grid(&output_grid);
    let tab = Tab {
        title: "Complete walk".to_string(),
        strings: vec![],
        grid: rows,
    };
    tabs.insert(1, tab);

    DayOutput {
        silver_output: format!("{}", visited_locations.len()),
        gold_output: format!("{}", blockage_locations.len()),
        diagnostic: Diagnostic::with_tabs(tabs, format!("errors: {:?}, history: {:?}, starting_position: {:?}", errors, history, starting_position)),
    }
}

fn apply_locations_to_output_grid<T: IntoIterator<Item=Coord>>(output_grid: &mut Vec<Vec<OutputLetter>>, locations: T, new_letter: OutputLetter) {
    for coord in locations.into_iter() {
        let letter = find_output_coord_mut(output_grid, coord);
        if let Some(letter) = letter {
            *letter = new_letter.clone();
        }
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

type Coord = (i32, i32);
type Grid = Vec<Vec<Letter>>;

fn check_for_loopability(grid: &Grid, initial_visited_locations: &HashMap<Coord, HashSet<Coord>>, starting_position: Coord, starting_dir: Coord) -> (bool, Vec<Vec<OutputLetter>>) {
    let mut history = Vec::new();
    let mut current_position = starting_position;
    let mut facing_dir = starting_dir;
    let mut visited_locations = initial_visited_locations.clone();
    let mut checked_locations = HashSet::new();
    let loopable = loop {
        let footsteps_here = visited_locations.entry(current_position).or_insert(HashSet::new());
        if footsteps_here.contains(&facing_dir) {
            // We've walked here, in the same direction, before
            break true;
        }
        footsteps_here.insert(facing_dir);
        let next_position = add_coord(current_position, facing_dir);
        let next_tile = find_coord(&grid, next_position);
        match next_tile {
            None => {
                if ENABLE_HISTORY {
                    history.push(format!("Break with current: {:?}, next: {:?}", current_position, next_position));
                }
                // We walked outside of bounds, we're done
                break false;
            }
            Some(tile_type) => {
                let dir_to_the_right = (-facing_dir.1, facing_dir.0);
                match tile_type {
                    Letter::Guard |
                    Letter::Dot => {
                        if ENABLE_HISTORY {
                            history.push(format!("Walk {:?} with current: {:?}, next: {:?}", facing_dir, current_position, next_position));
                        }
                        checked_locations.insert(next_position);
                        current_position = next_position;
                    }
                    Letter::Hash => {
                        if ENABLE_HISTORY {
                            history.push(format!("Turn {:?} with current: {:?}, next: {:?}", facing_dir, current_position, next_position));
                        }
                        facing_dir = dir_to_the_right;
                    }
                }
                // Check for loopability
            }
        }
    };
    if ENABLE_HISTORY {
        log::info!("Walking from {:?}, dir {:?}, result {:?}, history: {:?}", starting_position, starting_dir, loopable, history);
    }
    let mut output_grid = output_grid_from_grid(&grid);
    apply_locations_to_output_grid(&mut output_grid, initial_visited_locations.keys().map(|a|a.clone()), OutputLetter::Walked);
    apply_locations_to_output_grid(&mut output_grid, checked_locations.iter().map(|a|a.clone()), OutputLetter::Checked);
    apply_locations_to_output_grid(&mut output_grid, vec![starting_position].into_iter(), OutputLetter::CheckingStartLocation);
    (loopable, output_grid)
}

fn cells_from_output_grid(grid: &Vec<Vec<OutputLetter>>) -> Vec<Vec<GridCell>> {
    grid.iter().map(|row| {
        row.iter().map(|letter| {
            letter.to_cell()
        }).collect::<Vec<GridCell>>()
    }).collect::<Vec<Vec<GridCell>>>()
}

fn output_grid_from_grid(grid: &Vec<Vec<Letter>>) -> Vec<Vec<OutputLetter>> {
    grid.iter().map(|row| {
        row.iter().map(|letter| {
            letter.into()
        }).collect::<Vec<OutputLetter>>()
    }).collect::<Vec<Vec<OutputLetter>>>()
}

fn find_output_coord_mut(grid: &mut Vec<Vec<OutputLetter>>, coord: (i32, i32)) -> Option<&mut OutputLetter> {
    let (x, y): (Option<usize>, Option<usize>) = (coord.0.try_into().ok(), coord.1.try_into().ok());
    if let (Some(x), Some(y)) = (x, y) {
        grid.get_mut(y).map(|row| {
            row.get_mut(x)
        }).unwrap_or(None)
    } else {
        None
    }
}
