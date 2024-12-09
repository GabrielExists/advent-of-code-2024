use std::collections::HashSet;
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::grid::Grid;


pub fn puzzle(input: &str) -> DayOutput {
    let mut tabs = Vec::new();
    let mut grid = Grid::new();
    fn cell_function(cell: &Option<u64>) -> String {
        match cell {
            None => ".".to_string(),
            Some(num) => format!("{}", num),
        }
    }
    let (initial_disk_spaces, disk_spaces, errors) = puzzle_silver(input);
    grid.add_row_from(&initial_disk_spaces, cell_function);
    grid.add_row_from(&disk_spaces, cell_function);
    let silver_checksum = calculate_checksum(&disk_spaces);
    let (initial_disk_spaces, disk_spaces, errors) = puzzle_gold(input);
    grid.add_row_from(&initial_disk_spaces, cell_function);
    grid.add_row_from(&disk_spaces, cell_function);
    let gold_checksum = calculate_checksum(&disk_spaces);
    tabs.push(Tab {
        title: "Tab".to_string(),
        strings: vec![],
        grid: grid.to_tab_grid(),
    });
    DayOutput {
        silver_output: format!("{}", silver_checksum),
        gold_output: format!("{}", gold_checksum),
        diagnostic: Diagnostic::with_tabs(tabs, format!("errors: {:?}, disk_spaces {:?}", errors, disk_spaces)),
    }
}

fn puzzle_silver(input: &str) -> (Vec<Option<u64>>, Vec<Option<u64>>, Vec<String>) {
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
    (initial_disk_spaces, disk_spaces, errors)
}

#[derive(Clone, Debug)]
struct DiskSpace {
    slots: u64,
    id: Option<usize>,
}

fn puzzle_gold(input: &str) -> (Vec<Option<u64>>, Vec<Option<u64>>, Vec<String>) {
    let mut disk_spaces: Vec<DiskSpace> = Vec::new();
    let mut next_is_file = true;
    let mut next_file_id = 0;
    let mut errors = Vec::new();
    for character in input.chars().into_iter() {
        let num_slots = character.to_string().parse::<u64>();
        match num_slots {
            Err(parse_error) => {
                errors.push(format!("Failed to parse character '{}' with error {}", character, parse_error));
                break;
            }
            Ok(num_slots) => {
                let id = if next_is_file {
                    let file_id = next_file_id;
                    next_file_id += 1;
                    Some(file_id)
                } else {
                    None
                };
                disk_spaces.push(DiskSpace {
                    slots: num_slots,
                    id,
                });
                next_is_file = !next_is_file;
            }
        }
    }

    let files_to_move = disk_spaces.iter()
        .rev()
        .filter(|item|item.id.is_some())
        .map(|item|item.clone())
        .collect::<Vec<DiskSpace>>();

    // Copy files
    for file_to_move in files_to_move {
        for (index, slot) in disk_spaces.iter_mut().enumerate() {
            if slot.id.is_none() {
                if slot.slots > file_to_move.slots {
                    let extra_slots = slot.slots - file_to_move.slots;
                    slot.slots = file_to_move.slots;
                    slot.id = file_to_move.id;
                    disk_spaces.insert(index + 1, DiskSpace {
                        slots: extra_slots,
                        id: None,
                    });
                    break;
                } else if slot.slots == file_to_move.slots {
                    slot.id = file_to_move.id;
                    break;
                }
            }
        }
    }
    // Deduplicate
    let mut seen = HashSet::new();
    for disk_space in disk_spaces.iter_mut() {
        if let Some(id) = disk_space.id {
            if seen.contains(&id) {
                disk_space.id = None;
            } else {
                seen.insert(id);
            }
        }
    }

    let initial_disk_spaces = disk_spaces.clone();
    (split_files(initial_disk_spaces), split_files(disk_spaces), errors)
}

fn split_files(input: Vec<DiskSpace>) -> Vec<Option<u64>> {
    let mut output = Vec::new();
    for item in input.into_iter() {
        for _ in 0..item.slots {
            output.push(item.id.map(|id| id as u64));
        }
    }
    output
}

fn calculate_checksum(input: &Vec<Option<u64>>) -> u64 {
    input.iter().enumerate().fold(0, |acc, (index, cell)| {
        match cell {
            None => acc,
            Some(id) => acc + (index as u64 * (*id)),
        }
    })
}