use std::collections::HashSet;
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::grid::{Coord, Grid};

pub fn puzzle(input: &str) -> DayOutput {
    let input_grid = Grid::from_filtered(input, |character| {
        character.to_digit(10)
    });

    let grid = input_grid.clone();
    let mut sum = 0;
    let mut sum_rating = 0;
    let mut tuples = Vec::new();
    for coord in grid.get_all_coords().into_iter() {
        let (complete_trails, rating) = continue_trail(&grid, 0, coord);
        sum += complete_trails.len();
        sum_rating += rating;
        if !complete_trails.is_empty() {
            tuples.push(format!("{}: {}, {:?}", coord, rating, complete_trails))
        }
    }

    let tabs = vec![
        Tab {
            title: "Tab".to_string(),
            strings: tuples,
            grid: input_grid.to_tab_grid(),
        },
    ];
    DayOutput {
        silver_output: format!("{}", sum),
        gold_output: format!("{}", sum_rating),
        diagnostic: Diagnostic::with_tabs(tabs, format!("")),
    }
}

pub fn continue_trail(grid: &Grid<u32>, expected_height: u32, current_coord: Coord) -> (HashSet<Coord>, u32) {
    if let Some(current_height) = grid.get(current_coord) {
        if *current_height == expected_height {
            if *current_height == 9 {
                (HashSet::from([current_coord]), 1)
            } else {
                let mut trail_endings = HashSet::new();
                let mut rating = 0;
                for dir in Coord::get_orthagonal_dirs() {
                    let (current_trail_endings, current_rating) = continue_trail(&grid, expected_height + 1, current_coord.add(&dir));
                    trail_endings.extend(current_trail_endings.into_iter());
                    rating += current_rating;
                }
                (trail_endings, rating)
            }
        } else {
            (HashSet::new(), 0)
        }
    } else {
        (HashSet::new(), 0)
    }
}
