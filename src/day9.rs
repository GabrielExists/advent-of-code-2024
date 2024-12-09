use std::iter::zip;
use std::num::ParseIntError;
use regex::{Regex};
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::grid::Grid;


pub fn puzzle(input: &str) -> DayOutput {
    let mut disk_spaces: Vec<Option<u64>> = Vec::new();
    let mut next_is_file = true;
    let mut errors = Vec::new();
    let mut file_id = 0;
    for character in input.chars().into_iter() {
        let num_slots = character.to_string().parse::<u64>();
        match num_slots {
            Err(parse_error) => {
                errors.push(format!("Failed to parse character '{}' with error {}", character, parse_error));
                break;
            }
            Ok(num_slots) => {
                if next_is_file {
                    for _ in 0..num_slots {
                        disk_spaces.push(Some(file_id))
                    }
                    file_id += 1;
                } else {
                    for _ in 0..num_slots {
                        disk_spaces.push(None)
                    }
                }
                next_is_file = !next_is_file;
            }
        }
    }
    let initial_disk_spaces = disk_spaces.clone();

    let mut put_index = 0;
    let mut current_held = None;
    loop {
        match current_held {
            None => {
                match disk_spaces.pop() {
                    None => {
                        // the disk is empty, stop.
                        break;
                    }
                    Some(new_held) => {
                        current_held = new_held;
                    }
                }
            }
            Some(id) => {
                match disk_spaces.get_mut(put_index) {
                    None => {
                        // We're past the end of the array.
                        // This probably means we just popped the item in this slot,
                        // so we put it back and end.
                        disk_spaces.push(Some(id));
                        break;
                    }
                    Some(slot) => {
                        match slot {
                            None => {
                                // We found an empty slot, move the currently held object to here
                                *slot = current_held.take();
                            }
                            Some(_) => {
                                // We found an occupied slot, just keep moving to the right
                            }
                        }
                        put_index += 1;
                    }
                }
            }
        }
    }

    let mut tabs = Vec::new();
    let mut grid = Grid::new();
    let cell_function = |cell: &Option<u64>| {
        match cell {
            None => ".".to_string(),
            Some(num) => format!("{}", num),
        }
    };
    grid.add_row_from(&initial_disk_spaces, cell_function);
    grid.add_row_from(&disk_spaces, cell_function);
    tabs.push(Tab {
        title: "Tab".to_string(),
        strings: vec![],
        grid: grid.to_tab_grid(),
    });
    DayOutput {
        silver_output: format!("{}", calculate_checksum(&disk_spaces)),
        gold_output: format!("{}", 0),
        diagnostic: Diagnostic::with_tabs(tabs, format!("errors: {:?}, disk_spaces {:?}", errors, disk_spaces)),
    }
}

fn calculate_checksum(input: &Vec<Option<u64>>) -> u64 {
    input.iter().enumerate().fold(0, |acc, (index, cell)| {
        match cell {
            None => acc,
            Some(id) => acc + (index as u64 * (*id)),
        }
    })
}