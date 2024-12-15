use std::fmt::{Display, Formatter};
use std::ops::Deref;
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::grid::{Coord, Grid};

#[derive(PartialEq, Clone, Debug)]
enum Tile {
    Wall,
    Box,
    BoxLeft,
    BoxRight,
    Robot,
    Empty,
}

pub fn puzzle(input: &str) -> DayOutput {
    let mut split = input.split("\n\n");
    let mut errors = Vec::new();
    let mut tabs = vec![];
    let mut coordinate_sum = 0;
    let mut coordinate_sum_gold = 0;
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
        let starting_grid_gold = Grid::from_filtered_flatten(input_grid, |character| {
            match character {
                '#' => Some(vec![Tile::Wall, Tile::Wall]),
                'O' => Some(vec![Tile::BoxLeft, Tile::BoxRight]),
                '@' => Some(vec![Tile::Robot, Tile::Empty]),
                '.' => Some(vec![Tile::Empty, Tile::Empty]),
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
            title: "Start Grid Gold".to_string(),
            strings: vec![],
            grid: starting_grid_gold.to_tab_grid(),
        });
        tabs.push(Tab {
            title: "Input Commands".to_string(),
            strings: vec![format!("{:?}", movements)],
            grid: vec![],
        });

        let mut grid = starting_grid.clone();
        if let Some(mut robot_position) = grid.find(|tile| *tile == Tile::Robot) {
            for movement in movements.iter() {
                apply_movement(&mut grid, &mut robot_position, movement, true, &mut errors);
            }
        };
        coordinate_sum = calculate_gps_sum(&grid);
        let mut grid_gold = starting_grid_gold.clone();
        if let Some(mut robot_position) = grid_gold.find(|tile| *tile == Tile::Robot) {
            for movement in movements.iter() {
                let should_apply = apply_movement(&mut grid_gold, &mut robot_position, movement, false, &mut errors);
                if should_apply {
                    apply_movement(&mut grid_gold, &mut robot_position, movement, true, &mut errors);
                }
            }
        };
        coordinate_sum_gold = calculate_gps_sum(&grid_gold);
        tabs.insert(3, Tab {
            title: "Output Grid".to_string(),
            strings: vec![],
            grid: grid.to_tab_grid(),
        });
        tabs.insert(4, Tab {
            title: "Output Grid Gold".to_string(),
            strings: vec![],
            grid: grid_gold.to_tab_grid(),
        });
    }
    DayOutput {
        silver_output: format!("{}", coordinate_sum),
        gold_output: format!("{}", coordinate_sum_gold),
        diagnostic: Diagnostic::with_tabs(tabs, format!("Errors: {:?}", errors)),
    }
}

fn apply_movement(grid: &mut Grid<Tile>, position: &mut Coord, movement: &Coord, apply: bool, error: &mut Vec<String>) -> bool {
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
                    apply_movement(grid, &mut new_position.clone(), movement, apply, error)
                }
                Tile::BoxLeft => {
                    if movement.deref().0 == 0 {
                        let moved_left = apply_movement(grid, &mut new_position.clone(), movement, apply, error);
                        let moved_right = apply_movement(grid, &mut (new_position.clone().add(&Coord::new(1, 0))), movement, apply, error);
                        moved_left && moved_right
                    } else {
                        apply_movement(grid, &mut new_position.clone(), movement, apply, error)
                    }
                }
                Tile::BoxRight => {
                    if movement.deref().0 == 0 {
                        let moved_left = apply_movement(grid, &mut (new_position.clone().add(&Coord::new(-1, 0))), movement, apply, error);
                        let moved_right = apply_movement(grid, &mut new_position.clone(), movement, apply, error);
                        moved_left && moved_right
                    } else {
                        apply_movement(grid, &mut new_position.clone(), movement, apply, error)
                    }
                }
                Tile::Robot => false,
                Tile::Empty => true,
            };
            if should_move {
                if apply {
                    grid.swap(*position, new_position);
                    *position = new_position;
                }
                true
            } else {
                false
            }
        }
    }
}

fn calculate_gps_sum(grid: &Grid<Tile>) -> u64 {
    grid.map_grid(|tile, x, y| {
        if *tile == Tile::Box || *tile == Tile::BoxLeft {
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
            Tile::BoxLeft => f.write_str("["),
            Tile::BoxRight => f.write_str("]"),
            Tile::Robot => f.write_str("@"),
            Tile::Empty => f.write_str("."),
        }
    }
}