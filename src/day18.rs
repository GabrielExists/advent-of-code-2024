use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::grid::{Coord, Grid};

type Key = Coord;
type TileData = (u64, HashSet<Key>);
type CandidateMap = HashMap<Key, TileData>;

const WIDTH: usize = 71;
const HEIGHT: usize = 71;
const TAKE: usize = 1024;

#[derive(PartialEq, Clone, Debug)]
enum Tile {
    Wall,
    Empty,
    Path,
}

pub fn puzzle(input: &str) -> DayOutput {
    let input_grid = Grid::new_repeat(WIDTH, HEIGHT, Tile::Empty);
    let wall_coordinates = input.split("\n").filter_map(|line| {
        let mut split = line.split(",");
        let x = split.next();
        let y = split.next();
        double_parse(x, y).map(|(x, y)| {
            Coord::new(x as i32, y as i32)
        })
    }).collect();

    let mut tabs = vec![];

    let mut errors = vec![];
    let mut silver = 0;
    let mut gold = String::new();
    if let Ok(steps) = pathfind(&input_grid, &mut tabs, &wall_coordinates, TAKE) {
        silver = steps;
    }
    let mut front = TAKE;
    let mut end = wall_coordinates.len();
    while front + 1 != end {
        let middle = front + (end - front).div_ceil(2);
        errors.push(format!("{}, {}, {}", front, middle, end));
        match pathfind(&input_grid, &mut tabs, &wall_coordinates, middle) {
            Ok(_steps) => {
                front = middle;
            }
            Err(coord) => {
                end = middle;
                if let Some(coord) = coord {
                    gold = format!("{},{}", coord.deref().0, coord.deref().1);
                }
            }
        }
    }

    DayOutput {
        silver_output: format!("{}", silver),
        gold_output: format!("{}", gold),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
    }
}

fn pathfind(input_grid: &Grid<Tile>, mut tabs: &mut Vec<Tab>, wall_coordinates: &Vec<Coord>, num_to_take: usize) -> Result<usize, Option<Coord>> {
    let start = Coord::new(0, 0);
    let end = Coord::new(WIDTH as i32 - 1, HEIGHT as i32 - 1);
    let mut grid = input_grid.clone();
    let mut frontier: CandidateMap = create_frontier(&grid, start);
    let mut explored: CandidateMap = HashMap::new();
    for coord in wall_coordinates.iter().take(num_to_take) {
        if let Some(tile) = grid.get_mut(*coord) {
            *tile = Tile::Wall;
        }
    }

    while !frontier.is_empty() {
        if let Some((key, frontier_data)) = frontier.iter().next() {
            let key = key.clone();
            let frontier_data = frontier_data.clone();
            let frontier_cost = frontier_data.0;
            frontier.remove(&key);

            add_to_candidate_map(&mut explored, key, frontier_data);
            let source = key;

            for dir in Coord::get_orthagonal_dirs() {
                let new_position = source.add(&dir);
                let new_cost = frontier_cost + 1;
                let new_came_from = HashSet::from([key]);
                let (exists, exists_but_expensive) = if let Some((explored_cost, explored_came_from)) = explored.get_mut(&new_position) {
                    if *explored_cost == new_cost {
                        explored_came_from.extend(new_came_from.iter());
                    }
                    (true, *explored_cost > new_cost)
                } else {
                    (false, false)
                };
                let in_grid = Some(&Tile::Empty) == grid.get(new_position);
                if in_grid && (!exists || exists_but_expensive) {
                    add_to_candidate_map(&mut frontier, new_position, (new_cost, new_came_from));
                }
            }
        }
    }

    if let Some((end_key, _cost)) = get_end_tile(&mut explored, end) {
        let path_tiles: HashSet<Coord> = follow_path(&explored, end_key, true);
        add_tab_visited(&grid, &mut tabs, &path_tiles, format!("Path {}", num_to_take));
        Ok(path_tiles.len() - 1)
    } else {
        let coord = wall_coordinates.get(num_to_take - 1).map(|a| a.clone());
        add_tab_visited(&grid, &mut tabs, &HashSet::new(), format!("Nope {} {:?}", num_to_take, coord));
        Err(coord)
    }
}

fn create_frontier(grid: &Grid<Tile>, start: Coord) -> CandidateMap {
    let start_tile = grid.get(start);
    if let Some(Tile::Empty) = start_tile {
        HashMap::from_iter([(start, (0, HashSet::new()))].into_iter())
    } else {
        HashMap::new()
    }
}


fn add_to_candidate_map(map: &mut CandidateMap, key: Key, new_data: TileData) {
    match map.entry(key) {
        Entry::Occupied(mut occupied) => {
            combine_tile_data(occupied.get_mut(), &new_data);
        }
        Entry::Vacant(vacant) => {
            vacant.insert(new_data);
        }
    }
}

fn combine_tile_data(first: &mut TileData, second: &TileData) {
    let (first_cost, first_came_from) = first;
    let (second_cost, second_came_from) = second;
    if *second_cost < *first_cost {
        *first_came_from = second_came_from.clone();
        *first_cost = *second_cost;
    } else if *second_cost == *first_cost {
        first_came_from.extend(second_came_from.iter());
    }
}

fn get_end_tile(explored: &mut CandidateMap, end: Coord) -> Option<(Key, u64)> {
    explored.get(&end).map(|tile_data| (end, tile_data.0))
}

fn follow_path(map: &CandidateMap, end_key: Key, only_shortest: bool) -> HashSet<Coord> {
    let mut remaining_tiles = HashSet::from([end_key]);
    let mut path_tiles = HashSet::new();
    while !remaining_tiles.is_empty() {
        if let Some(next_key) = remaining_tiles.iter().next() {
            let next_key = next_key.clone();
            remaining_tiles.take(&next_key);
            path_tiles.insert(next_key);
            if let Some((_cost, came_from)) = map.get(&next_key) {
                if only_shortest {
                    remaining_tiles.extend(came_from.iter().take(1));
                } else {
                    remaining_tiles.extend(came_from.iter());
                }
            }
        }
    }
    path_tiles
}

fn add_tab_visited(input_grid: &Grid<Tile>, tabs: &mut Vec<Tab>, path_tiles: &HashSet<Coord>, title: String) {
    let mut grid = input_grid.clone();
    // Apply the path tiles
    for pos in path_tiles.iter() {
        if let Some(tile_handle) = grid.get_mut(*pos) {
            *tile_handle = Tile::Path;
        }
    }
    tabs.push(Tab {
        title,
        strings: vec![],
        grid: grid.to_tab_grid(),
    });
}


impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Tile::Wall => f.write_str("#"),
            Tile::Empty => f.write_str("."),
            Tile::Path => f.write_str("o"),
        }
    }
}

pub fn double_parse(first: Option<&str>, second: Option<&str>) -> Option<(usize, usize)> {
    match (
        first.map(|item| item.parse::<usize>()),
        second.map(|item| item.parse::<usize>())
    ) {
        (
            Some(Ok(first)),
            Some(Ok(second))
        ) => Some((first, second)),
        _ => None,
    }
}
