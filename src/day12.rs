use std::collections::HashSet;
use std::ops::Deref;
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::grid::{Coord, Grid};

pub fn puzzle(input: &str) -> DayOutput {
    let input_grid = Grid::from(input, |character| {
        character
    });
    let mut tabs = vec![];

    let grid = input_grid.clone();
    let mut sum_silver = 0;
    let mut sum_gold = 0;
    let mut used_coords = HashSet::new();
    let mut diagnostic_strings = Vec::new();
    for coord in grid.get_all_coords().into_iter() {
        if !used_coords.contains(&coord) {
            // Fence segments contain the start and end corners of fence segments, represented by the tile that has that corner in the top left.
            if let Some(character) = grid.get(coord) {
                let (area, fence_segments) = traverse(&grid, &mut used_coords, coord, None, *character);
                let boundary = fence_segments.len() as u64;
                sum_silver += area * boundary;
                let fence_segment_strings = fence_segments.iter().map(|(first, second)| {
                    format!("{}: {}, {}", character, first, second)
                }).collect::<Vec<_>>();
                let sides = combine_fences(fence_segments);
                sum_gold += area * sides.len() as u64;
                diagnostic_strings.push(format!("Area with {} starting at {} had area {} and boundary {}, sides {}", *character, coord, area, boundary, sides.len()));
                let strings = fence_segment_strings.into_iter().chain(sides.into_iter().map(|(first, second)| {
                    format!("Side {}, {}", first, second)
                })).collect();
                tabs.push(Tab {
                    title: format!("Sides {}", tabs.len()),
                    strings,
                    grid: vec![],
                })
            }
        }
    }

    tabs.insert(0, Tab {
        title: "Input grid".to_string(),
        strings: diagnostic_strings,
        grid: input_grid.to_tab_grid(),
    },
    );
    DayOutput {
        silver_output: format!("{}", sum_silver),
        gold_output: format!("{}", sum_gold),
        diagnostic: Diagnostic::with_tabs(tabs, format!("")),
    }
}

fn traverse(grid: &Grid<char>, used_coords: &mut HashSet<Coord>, coord: Coord, previous_coord: Option<Coord>, area_char: char) -> (u64, HashSet<(Coord, Coord)>) {
    if let Some(current_char) = grid.get(coord) {
        if area_char == *current_char {
            if !used_coords.contains(&coord) {
                used_coords.insert(coord);
                let mut area = 1;
                let mut boundary = HashSet::new();
                for dir in Coord::get_orthagonal_dirs().into_iter() {
                    let adjacent_coord = coord.add(&dir);
                    let (new_area, new_boundary) = traverse(grid, used_coords, adjacent_coord, Some(coord), area_char);
                    area += new_area;
                    boundary.extend(new_boundary.into_iter());
                }
                (area, boundary)
            } else {
                (0, HashSet::new())
            }
        } else {
            (0, get_fence(coord, previous_coord))
        }
    } else {
        (0, get_fence(coord, previous_coord))
    }
}

fn get_fence(coord: Coord, previous_coord: Option<Coord>) -> HashSet<(Coord, Coord)> {
    if let Some(previous_coord) = previous_coord {
        if previous_coord.0.0 == coord.0.0 {
            let x = previous_coord.0.0;
            let y = std::cmp::max(previous_coord.deref().1, coord.deref().1);
            // Make inside and outside edges different so independent segments that meet in a cross don't combine
            if previous_coord.deref().1 < coord.deref().1 {
                HashSet::from([(
                    Coord::new(x, y),
                    Coord::new(x + 1, y)
                )])
            } else {
                HashSet::from([(
                    Coord::new(x + 1, y),
                    Coord::new(x, y),
                )])
            }
        } else {
            let x = std::cmp::max(previous_coord.deref().0, coord.deref().0);
            let y = previous_coord.0.1;
            if previous_coord.deref().0 < coord.deref().0 {
                HashSet::from([(
                    Coord::new(x, y),
                    Coord::new(x, y + 1)
                )])
            } else {
                HashSet::from([(
                    Coord::new(x, y + 1),
                    Coord::new(x, y),
                )])
            }
        }
    } else {
        HashSet::new()
    }
}


fn combine_fences(input: HashSet<(Coord, Coord)>) -> HashSet<(Coord, Coord)> {
    let mut output: Vec<(Coord, Coord, bool)> = Vec::new();
    for (mut new_start, mut new_end) in input.into_iter() {
        for (current_start, current_end, current_delete) in output.iter_mut() {
            if !*current_delete {
                if new_start.0.0 == new_end.0.0 && new_start.0.0 == current_start.0.0 && new_start.0.0 == current_end.0.0 {
                    if *current_start == new_end {
                        new_end = *current_end;
                        *current_delete = true;
                    } else if *current_end == new_start {
                        new_start = *current_start;
                        *current_delete = true;
                    }
                } else if new_start.0.1 == new_end.0.1 && new_start.0.1 == current_start.0.1 && new_start.0.1 == current_end.0.1 {
                    if *current_start == new_end {
                        new_end = *current_end;
                        *current_delete = true;
                    } else if *current_end == new_start {
                        new_start = *current_start;
                        *current_delete = true;
                    }
                }
            }
        }
        output.push((new_start, new_end, false));
    }
    output.into_iter().filter_map(|(start, end, delete)| {
        if delete {
            None
        } else {
            Some((start, end))
        }
    }).collect()
}
