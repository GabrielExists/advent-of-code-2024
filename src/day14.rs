use std::collections::HashSet;
use std::ops::Deref;
use std::str::FromStr;
use regex::{Captures, Regex};
use crate::app::{DayOutput, Diagnostic, Tab};
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

pub fn capture_parse<F: FromStr>(captures: &Captures, name: &str) -> Option<F> {
    captures.name(name).map(|s| s.as_str().parse::<F>().ok()).unwrap_or(None)
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
    let mut grid: Grid<u64> = Grid::new_repeat(GRID_WIDTH as usize, GRID_HEIGHT as usize, 0);
    for robot in robots.iter() {
        let x = (robot.px + robot.vx * SECONDS) % GRID_WIDTH;
        let y = (robot.py + robot.vy * SECONDS) % GRID_HEIGHT;
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
    // let input_grid = Grid::from(input, |character| {
    //     character
    // });
    let mut tabs = vec![];

    let mut silver = calculate_safety_score(&grid);
    let mut gold = 0;
    // let mut used_coords = HashSet::new();
    // let mut diagnostic_strings = Vec::new();

    tabs.insert(0, Tab {
        title: "Input".to_string(),
        strings: robots.iter().map(|robot|format!("{:?}", robot)).collect(),
        grid: vec![],
    });
    tabs.insert(0, Tab {
        title: "Grid".to_string(),
        strings: vec![],
        grid: grid.to_tab_grid(),
    });
    DayOutput {
        silver_output: format!("{}", silver),
        gold_output: format!("{}", gold),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
    }
}


fn calculate_safety_score(grid: &Grid<u64>) -> u64 {
    let mut quadrants = [0, 0, 0, 0];
    for coord in grid.get_all_coords() {
        if let Some(num_robots) = grid.get(coord) {
            let quadrant = if coord.deref().0 * 2 + 1 < GRID_WIDTH {
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
            };
        }
    }
    quadrants.into_iter().product()
}