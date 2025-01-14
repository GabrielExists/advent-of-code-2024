use std::ops::Deref;
use regex::Regex;
use yew::classes;
use crate::app::{class_string, DayOutput, Diagnostic, Tab};
use crate::common::capture_parse;
use crate::grid::{Coord, Grid};


const GRID_WIDTH: i32 = 101;
const GRID_HEIGHT: i32 = 103;
const SECONDS: i32 = 100;

#[derive(Clone, Debug)]
struct Robot {
    px: i32,
    py: i32,
    vx: i32,
    vy: i32,
}

pub fn puzzle(input: &str) -> DayOutput {
    let re = Regex::new(r"p\=(?<px>\d+),(?<py>\d+) v=(?<vx>-?\d+),(?<vy>-?\d+)").unwrap();
    let robots: Vec<Robot> = re.captures_iter(input).into_iter().filter_map(|captures| {
        match (
            capture_parse(&captures, "px"),
            capture_parse(&captures, "py"),
            capture_parse(&captures, "vx"),
            capture_parse(&captures, "vy"),
        ) {
            (Some(px), Some(py), Some(vx), Some(vy)) => {
                Some(Robot{px, py, vx ,vy})
            }
            _ => None
        }
    }).collect();

    let mut errors = Vec::new();
    // let input_grid = Grid::from(input, |character| {
    //     character
    // });
    let mut tabs = vec![];

    let silver_grid = apply_movement(&robots, SECONDS, &mut errors);
    let silver = calculate_safety_score(&silver_grid);
    // By manually searching, we find that there is a vertical constellation at 68, 169, 270
    // As well as a horizontal constellation at 136, 239, 342.
    // These are constant spacings of 101 and 103.
    // The point at which these intersect might be interesting.
    // So I solved y = 101x + 68, y = 103z + 136 and found that there is a y at 7037 and then again every additional 10403 seconds

    // For my second set of inputs I found vertical constellations at 77, 178, for a spacing of 101
    // And horizontal constellations at 18, 121, for a spacing of 103
    // y = 101x + 77, y = 103z + 18
    // y = 8258 + 10403 n
    let grid = apply_movement(&robots, 7037, &mut errors);
    add_tab(&mut tabs, 7037, grid);
    let grid = apply_movement(&robots, 17440, &mut errors);
    add_tab(&mut tabs, 17440, grid);
    let starting_point = 8258;
    let repeat_every = 10403;
    let grid = apply_movement(&robots, starting_point, &mut errors);
    add_tab(&mut tabs, starting_point, grid);
    let grid = apply_movement(&robots, starting_point + repeat_every, &mut errors);
    add_tab(&mut tabs, starting_point + repeat_every, grid);
    // Manual search data
    for seconds in 0..400 {
        let grid = apply_movement(&robots, seconds, &mut errors);
        add_tab(&mut tabs, seconds, grid);
    }

    // let mut used_coords = HashSet::new();
    // let mut diagnostic_strings = Vec::new();

    tabs.insert(0, Tab {
        title: "Input".to_string(),
        strings: robots.iter().map(|robot|format!("{:?}", robot)).collect(),
        grid: vec![],
    });
    tabs.insert(1, Tab {
        title: "Grid".to_string(),
        strings: vec![],
        grid: silver_grid.to_tab_grid(),
    });
    DayOutput {
        silver_output: format!("{}", silver),
        gold_output: format!("{}", 7037),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
    }
}

fn add_tab(tabs: &mut Vec<Tab>, seconds: i32, grid: Grid<u64>) {
    tabs.push(Tab {
        title: format!("G{}", seconds),
        strings: vec![],
        grid: grid.to_tab_grid_class(|num| if *num > 0 {
            class_string("bg-slate-300 text-slate-900")
        } else {
            classes!("")
        }),
    });
}

fn apply_movement(robots: &Vec<Robot>, seconds: i32, errors: &mut Vec<String>) -> Grid<u64> {
    let mut grid: Grid<u64> = Grid::new_repeat(GRID_WIDTH as usize, GRID_HEIGHT as usize, 0);
    for robot in robots.iter() {
        let x = (robot.px + robot.vx * seconds) % GRID_WIDTH;
        let y = (robot.py + robot.vy * seconds) % GRID_HEIGHT;
        let x = if x < 0 {
            GRID_WIDTH + x
        } else {
            x
        };
        let y = if y < 0 {
            GRID_HEIGHT + y
        } else {
            y
        };
        if let Some(tile) = grid.get_mut(Coord::new(x, y)) {
            *tile += 1;
        } else {
            errors.push(format!("Failed to get coordinate x {}, y {}", x, y));
        }
    }
    grid
}


fn calculate_safety_score(grid: &Grid<u64>) -> u64 {
    let mut quadrants = [0, 0, 0, 0];
    for coord in grid.get_all_coords() {
        if let Some(num_robots) = grid.get(coord) {
            if coord.deref().0 * 2 + 1 < GRID_WIDTH {
                if coord.deref().1 * 2 + 1 < GRID_HEIGHT {
                    quadrants[0] += *num_robots;
                } else if coord.deref().1 * 2 + 1 > GRID_HEIGHT {
                    quadrants[1] += *num_robots;
                }
            } else if coord.deref().0 * 2 + 1 > GRID_WIDTH {
                if coord.deref().1 * 2 + 1 < GRID_HEIGHT {
                    quadrants[2] += *num_robots;
                } else if coord.deref().1 * 2 + 1 > GRID_HEIGHT {
                    quadrants[3] += *num_robots;
                }
            }
        }
    }
    quadrants.into_iter().product()
}