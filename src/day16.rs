use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::grid::{Coord, Grid};

type CandidateMap = HashMap<(Coord, Coord), TileData>;

type TileData = (u64, HashSet<Coord>);

#[derive(PartialEq, Clone, Debug)]
enum Tile {
    Wall,
    Empty,

    Up(TileData),
    Right(TileData),
    Left(TileData),
    Down(TileData),
    Frontier(TileData),
    Error,
}

pub fn puzzle(input: &str) -> DayOutput {
    let mut start = None;
    let mut end = None;
    let input_grid = Grid::from_with_index_filtered(input, |character, x, y| {
        match character {
            '#' => Some(Tile::Wall),
            '.' => Some(Tile::Empty),
            'S' => {
                start = Some(Coord::new(x, y));
                Some(Tile::Empty)
            }
            'E' => {
                end = Some(Coord::new(x, y));
                Some(Tile::Empty)
            }
            _ => None,
        }
    });
    let mut tabs = vec![
        Tab {
            title: "Tab".to_string(),
            strings: vec![],
            grid: input_grid.to_tab_grid(),
        },
    ];

    let mut silver = None;
    let mut gold = 0;
    if let (Some(start), Some(end)) = (start, end) {
        let mut frontier: CandidateMap = create_frontier(&input_grid, start);
        let mut explored: CandidateMap = HashMap::new();
        // for step in 0..2000 {
        let mut step = 0;
        while !frontier.is_empty() {
            if let Some((key, (frontier_cost, frontier_came_from))) = frontier.iter().next() {
                let key = key.clone();
                let frontier_cost = frontier_cost.clone();
                let frontier_came_from = frontier_came_from.clone();
                frontier.remove(&key);
                let (source, dir) = key;

                let (explored_cost, explored_came_from) = explored.entry((source, dir)).or_insert((frontier_cost, frontier_came_from.clone()));
                if frontier_cost < *explored_cost {
                    *explored_cost = frontier_cost;
                    *explored_came_from = frontier_came_from;
                } else if frontier_cost == *explored_cost {
                    explored_came_from.extend(frontier_came_from.into_iter());
                }

                for (new_dir, new_cost) in [
                    (dir, frontier_cost + 1),
                    (dir.rotate_left(), frontier_cost + 1001),
                    (dir.rotate_right(), frontier_cost + 1001),
                ] {
                    let new_position = source.add(&new_dir);
                    let new_came_from = HashSet::from( [source]);
                    let (exists, exists_but_expensive) = if let Some((explored_cost, explored_came_from)) = explored.get_mut(&(new_position, new_dir)) {
                        if *explored_cost == new_cost {
                            explored_came_from.extend(new_came_from.iter());
                        }
                        (true, *explored_cost > new_cost)
                    } else {
                        (false, false)
                    };
                    let in_grid = Some(&Tile::Empty) == input_grid.get(new_position);
                    if in_grid && (!exists || exists_but_expensive) {
                        let frontier_data = frontier.entry((new_position, new_dir)).or_insert((new_cost, new_came_from.clone()));
                        let (frontier_cost, frontier_came_from) = frontier.entry((new_position, new_dir)).or_insert((new_cost, new_came_from.clone()));
                        if new_cost < *frontier_cost {
                            *frontier_cost = new_cost;
                            *frontier_came_from = new_came_from;
                        } else if new_cost == *frontier_cost {
                            frontier_came_from.extend(new_came_from.into_iter());
                        }
                    }
                }
                if step < 200 {
                    add_tab(&input_grid, &mut tabs, &frontier, &explored, format!("S{}", step), source, dir);
                }
                step += 1;
            }
        }
        add_tab(&input_grid, &mut tabs, &frontier, &explored, format!("Final state, {} steps", step), Coord::new(0, 0), Coord::new(0, 0));

        for dir in Coord::get_orthagonal_dirs().into_iter() {
            if let Some((cost, _came_from)) = explored.get(&(end, dir)) {
                match silver {
                    None => {
                        silver = Some(*cost);
                    }
                    Some(silver_cost) => {
                        if *cost < silver_cost {
                            silver = Some(*cost);
                        }
                    }
                }
            }
        }

        tabs.insert(1, Tab {
            title: "Frontier".to_string(),
            strings: frontier.iter().map(|a| format!("{:?}", a)).collect(),
            grid: vec![],
        });
        tabs.insert(2, Tab {
            title: "Explored".to_string(),
            strings: explored.iter().map(|a| format!("{:?}", a)).collect(),
            grid: vec![],
        });
    }


    DayOutput {
        silver_output: format!("{}", silver.unwrap_or(0)),
        gold_output: format!("{}", gold),
        diagnostic: Diagnostic::with_tabs(tabs, format!("")),
    }
}

fn add_tab(input_grid: &Grid<Tile>, tabs: &mut Vec<Tab>, frontier: &CandidateMap, explored: &CandidateMap, title: String, source: Coord, dir: Coord) {
    let mut grids = HashMap::new();
    for dir in Coord::get_orthagonal_dirs() {
        grids.insert(dir, input_grid.clone());
    }
    for ((explored_pos, explored_dir), tile_data) in explored.iter() {
        grids.get_mut(explored_dir).map(|grid| {
            if let Some(tile_handle) = grid.get_mut(*explored_pos) {
                let tile_data = tile_data.clone();
                let tile = match explored_dir.deref() {
                    (1, 0) => Tile::Right(tile_data),
                    (0, 1) => Tile::Down(tile_data),
                    (-1, 0) => Tile::Left(tile_data),
                    (0, -1) => Tile::Up(tile_data),
                    _ => Tile::Error,
                };
                *tile_handle = tile;
            }
        });
    }
    for ((frontier_pos, frontier_dir), tile_data) in frontier.iter() {
        grids.get_mut(frontier_dir).map(|grid| {
            if let Some(tile_handle) = grid.get_mut(*frontier_pos) {
                let tile_data = tile_data.clone();
                *tile_handle = Tile::Frontier(tile_data);
            }
        });
    }
    let mut grid = Grid::new();
    if let Some((first, rest)) = Coord::get_orthagonal_dirs().split_first() {
        if let Some(mut mushed_grid) = grids.remove(first) {
            let mut appended_grid = mushed_grid.clone();
            for coord in rest {
                if let Some(new_grid) = grids.remove(coord) {
                    mushed_grid.mush(&new_grid, |first, second| {
                        match (first.get_data_mut(), second.get_data()) {
                            (
                                Some((first_cost, first_came_from)),
                                Some((second_cost, second_came_from))
                            ) => {
                                let cost = std::cmp::min(*first_cost, *second_cost);
                                let mut came_from = first_came_from.clone();
                                came_from.extend(second_came_from.clone());
                                *first = Tile::Frontier((cost, came_from));
                            }
                            (Some(_), None) => {}
                            (None, Some(_)) => {
                                *first = second.clone();
                            }
                            (None, None) => {}
                        }
                    });
                    appended_grid.append(new_grid);
                }
            }
            grid.append(mushed_grid);
            grid.append(appended_grid);
        }
    }
    tabs.push(Tab {
        title,
        strings: vec![format!("{} {}", source, dir)],
        grid: grid.to_tab_grid_title(|cell| {
            match cell.clone() {
                Tile::Wall => String::new(),
                Tile::Empty => String::new(),
                Tile::Up(tile_data) |
                Tile::Right(tile_data) |
                Tile::Left(tile_data) |
                Tile::Down(tile_data) |
                Tile::Frontier(tile_data) => {
                    let (cost, came_from) = tile_data;
                    format!("{}, {:?}", cost, came_from)
                }
                Tile::Error => String::new(),
            }
        }),
    })
}

fn create_frontier(grid: &Grid<Tile>, start: Coord) -> CandidateMap {
    let start_tile = grid.get(start);
    if let Some(Tile::Empty) = start_tile {
        HashMap::from_iter([((start, Coord::new(1, 0)), (0, HashSet::new()))].into_iter())
    } else {
        HashMap::new()
    }
}


impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Tile::Wall => f.write_str("#"),
            Tile::Empty => f.write_str("."),
            Tile::Up(_) => f.write_str("^"),
            Tile::Right(_) => f.write_str(">"),
            Tile::Left(_) => f.write_str("<"),
            Tile::Down(_) => f.write_str("v"),
            Tile::Frontier(_) => f.write_str("o"),
            Tile::Error => f.write_str("!"),
        }
    }
}

fn combine_tile_data(first: &mut TileData, second: &TileData) {
    let (first_cost, first_came_from) = first;
    let (second_cost, second_came_from) = second;
    if *second_cost < *first_cost {
        *first_came_from = second_came_from.clone();
    } else if *second_cost == *first_cost {
        first_came_from.extend(second_came_from.iter());
    }
}

impl Tile {
    fn get_data(&self) -> Option<&TileData> {
        match self {
            Tile::Wall => None,
            Tile::Empty => None,
            Tile::Up(tile_data) => Some(tile_data),
            Tile::Right(tile_data) => Some(tile_data),
            Tile::Left(tile_data) => Some(tile_data),
            Tile::Down(tile_data) => Some(tile_data),
            Tile::Frontier(tile_data) => Some(tile_data),
            Tile::Error => None
        }
    }
    fn get_data_mut(&mut self) -> Option<&mut TileData> {
        match self {
            Tile::Wall => None,
            Tile::Empty => None,
            Tile::Up(tile_data) => Some(tile_data),
            Tile::Right(tile_data) => Some(tile_data),
            Tile::Left(tile_data) => Some(tile_data),
            Tile::Down(tile_data) => Some(tile_data),
            Tile::Frontier(tile_data) => Some(tile_data),
            Tile::Error => None
        }
    }
}