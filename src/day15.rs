use std::fmt::{Display, Formatter};
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::grid::{Coord, Grid};

#[derive(PartialEq, Clone, Debug)]
enum Tile {
    Wall,
    Box,
    Robot,
    Empty,
}

enum TileGold {
    Wall,
    BoxLeft,
    BoxRight,
    Robot,
    Empty
}

pub fn puzzle(input: &str) -> DayOutput {
    let mut split = input.split("\n\n");
    let mut errors = Vec::new();
    let mut tabs = vec![];
    let mut coordinate_sum = 0;
    if let (Some(input_grid), Some(input_movements)) = (split.next(), split.next()) {
        let starting_grid = Grid::from_filtered(input_grid, |character| {
            match character {
                '#' => Some(Tile::Wall),
                'O' => Some(Tile::Box),
                '@' => Some(Tile::Robot),
                '.' => Some(Tile::Empty),
                invalid => {
                    errors.push(format!("Found invalid tile '{}'", invalid));
                    None
                }
            }
        });
        let movements = input_movements.chars().filter_map(|character| {
            match character {
                '^' => Some(Coord::new(0, -1)),
                '>' => Some(Coord::new(1, 0)),
                'v' => Some(Coord::new(0, 1)),
                '<' => Some(Coord::new(-1, 0)),
                '\n' => None,
                invalid => {
                    errors.push(format!("Found invalid command '{}'", invalid));
                    None
                }
            }
        }).collect::<Vec<Coord>>();

        tabs.push(Tab {
            title: "Start Grid".to_string(),
            strings: vec![],
            grid: starting_grid.to_tab_grid(),
        });
        tabs.push(Tab {
            title: "Input Commands".to_string(),
            strings: vec![format!("{:?}", movements)],
            grid: vec![],
        });

        let mut grid = starting_grid.clone();
        if let Some(mut robot_position) = grid.find(|tile| *tile == Tile::Robot) {
            for movement in movements.iter() {
                apply_movement(&mut grid, &mut robot_position, movement, &mut errors);
            }
        };
        tabs.insert(2, Tab {
            title: "Output Grid".to_string(),
            strings: vec![],
            grid: grid.to_tab_grid(),
        });
        coordinate_sum = calculate_gps_sum(&grid);
    }
    DayOutput {
        silver_output: format!("{}", coordinate_sum),
        gold_output: format!("{}", 0),
        diagnostic: Diagnostic::with_tabs(tabs, format!("Errors: {:?}", errors)),
    }
}

fn apply_movement(grid: &mut Grid<Tile>, position: &mut Coord, movement: &Coord, error: &mut Vec<String>) -> bool {
    let new_position = position.add(movement);
    match grid.get(new_position) {
        None => {
            false
        }
        Some(target_tile) => {
            let target_tile = target_tile.clone();
            let should_move = match target_tile {
                Tile::Wall => false,
                Tile::Box => {
                    let mut moved_position = new_position;
                    apply_movement(grid, &mut moved_position, movement, error)
                }
                Tile::Robot => false,
                Tile::Empty => true,
            };
            if should_move {
                let moved = grid.swap(*position, new_position);
                *position = new_position;
                moved
            } else {
                false
            }
        }
    }
}

fn calculate_gps_sum(grid: &Grid<Tile>) -> u64 {
    grid.map_grid(|tile, x, y| {
        if *tile == Tile::Box {
            y * 100 + x
        } else {
            0
        }
    }).into_iter().map(|row| {
        row.into_iter().map(|int| int as u64).sum::<u64>()
    }).sum()
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Tile::Wall => f.write_str("#"),
            Tile::Box => f.write_str("O"),
            Tile::Robot => f.write_str("@"),
            Tile::Empty => f.write_str("."),
        }
    }
}