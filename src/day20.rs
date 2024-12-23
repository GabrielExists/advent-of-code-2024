use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::fmt::{Display, Formatter};
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::grid::{Coord, Grid};

type Key = Coord;
type TileData = (u64, HashMap<Key, u64>);
type CandidateMap = HashMap<Key, TileData>;

const LIMIT: u64 = 100;

#[derive(PartialEq, Clone, Debug)]
enum Tile {
    Wall,
    Empty,
    Path,
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

    let mut tabs = vec![];
    let mut errors: Vec<String> = vec![];
    let mut silver = 0;
    let mut gold = String::new();
    tabs.push(Tab {
        title: "Input".to_string(),
        strings: vec![],
        grid: input_grid.to_tab_grid(),
    });
    if let (Some(start), Some(end)) = (start, end) {
        let explored = pathfind(&input_grid, &mut tabs, start, end);
        let (path_tiles, default_total_steps) = follow_path(&explored, end, true);
        add_tab_path(&mut tabs, &input_grid, path_tiles, default_total_steps, format!("Default"));

        let mut cheats: Vec<((Coord, Coord), u64)> = Vec::new();
        for cheat_from in explored.keys() {
            for cheat_to in explored.keys() {
                let steps = steps_away(cheat_from, cheat_to);
                if steps <= 2 && cheat_to != cheat_from {
                    // if let Some((end_key, _cost)) = explored.get(&end).map(|tile_data| (end, tile_data.0)) {
                    //     let path_tiles: HashSet<Coord> = follow_path_with_cheat(&explored, end_key, true);
                    // }
                    // add_tab_path(&input_grid, &explored, end, &mut tabs, format!("F{} {}", cheat_from, cheat_to));
                    let mut explored = explored.clone();
                    if let Entry::Occupied(data_to) = explored.entry(*cheat_to) {
                        let cost_to = data_to.get().0 + steps as u64;
                        if let Entry::Occupied(mut data_from) = explored.entry(*cheat_from) {
                            let data_from = data_from.get_mut();
                            combine_tile_data(data_from, &(cost_to, HashMap::from([(*cheat_to, steps as u64)])));
                            let (path_tiles, total_steps) = follow_path(&explored, end, true);
                            // add_tab_path(&mut tabs, &input_grid, path_tiles, total_steps, format!("F{} {}", cheat_from, cheat_to));
                            cheats.push(((*cheat_from, *cheat_to), total_steps));
                        }
                    }
                }
            }
        }
        tabs.push(Tab {
            title: "Cheats".to_string(),
            strings: cheats.iter().filter_map(|cheat| {
                if cheat.1 != default_total_steps {
                    Some(format!("{:?}", cheat))
                } else {
                    None
                }
            }).collect(),
            grid: vec![],
        });
        let mut saved_to_num = HashMap::new();
        for (_key, total_cost) in cheats.iter() {
            let cost_diff = default_total_steps - total_cost;
            let mut num = saved_to_num.entry(cost_diff).or_insert(0);
            *num += 1;
            if cost_diff >= LIMIT {
                silver += 1;
            }
        }

        tabs.push(Tab {
            title: "Cheat summary".to_string(),
            strings: saved_to_num.iter().map(|(saved, num)| {
                format!("saved: {}, num: {}", saved, num)
            }).collect(),
            grid: vec![],
        });
    }
    // let mut front = TAKE;
    // let mut end = wall_coordinates.len();
    // while front + 1 != end {
    //     let middle = front + (end - front).div_ceil(2);
    //     errors.push(format!("{}, {}, {}", front, middle, end));
    //     match pathfind(&input_grid, &mut tabs) {
    //         Ok(_steps) => {
    //             front = middle;
    //         }
    //         Err(coord) => {
    //             end = middle;
    //             if let Some(coord) = coord {
    //                 gold = format!("{},{}", coord.deref().0, coord.deref().1);
    //             }
    //         }
    //     }
    // }

    DayOutput {
        silver_output: format!("{}", silver),
        gold_output: format!("{}", gold),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
    }
}

fn steps_away(start: &Coord, end: &Coord) -> i32 {
    i32::abs(start.0.0 - end.0.0) + i32::abs(start.0.1 - end.0.1)
}

fn add_tab_path(tabs: &mut Vec<Tab>, grid: &Grid<Tile>, path_tiles: HashSet<Coord>, total_cost: u64, title: String) {
    let mut grid = grid.clone();
    // Apply the path tiles
    for pos in path_tiles.iter() {
        if let Some(tile_handle) = grid.get_mut(*pos) {
            *tile_handle = Tile::Path;
        }
    }
    tabs.push(Tab {
        title,
        strings: vec![format!("{} nanos", total_cost)],
        grid: grid.to_tab_grid(),
    });
}

fn pathfind(input_grid: &Grid<Tile>, mut tabs: &mut Vec<Tab>, start: Coord, end: Coord) -> CandidateMap {
    let mut grid = input_grid.clone();
    let mut frontier: CandidateMap = create_frontier(&grid, start);
    let mut explored: CandidateMap = HashMap::new();

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
                let new_came_from = HashMap::from([(key, 1)]);
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

    explored
    // if let Some((end_key, _cost)) = get_end_tile(&mut explored, end) {
    //     let path_tiles: HashSet<Coord> = follow_path(&explored, end_key, true);
    //     add_tab_visited(&grid, &mut tabs, &path_tiles, format!("Path"));
    //     Ok(path_tiles.len() - 1)
    // } else {
    //     add_tab_visited(&grid, &mut tabs, &HashSet::new(), format!("Nope"));
    //     Err(())
    // }
}

fn create_frontier(grid: &Grid<Tile>, start: Coord) -> CandidateMap {
    let start_tile = grid.get(start);
    if let Some(Tile::Empty) = start_tile {
        HashMap::from_iter([(start, (0, HashMap::new()))].into_iter())
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

fn follow_path(map: &CandidateMap, end: Key, only_shortest: bool) -> (HashSet<Coord>, u64) {
    if let Some((end_key, _cost)) = map.get(&end).map(|tile_data| (end, tile_data.0)) {
        let mut remaining_tiles = HashMap::from([(end_key, 0)]);
        let mut path_tiles = HashSet::<Coord>::new();
        let mut total_steps = 0;
        while !remaining_tiles.is_empty() {
            if let Some((next_key, steps)) = remaining_tiles.iter().next() {
                let next_key = next_key.clone();
                let steps = steps.clone();
                remaining_tiles.remove(&next_key);
                path_tiles.insert(next_key);
                total_steps += steps;
                if let Some((_cost, came_from)) = map.get(&next_key) {
                    if only_shortest {
                        remaining_tiles.extend(came_from.iter().take(1));
                    } else {
                        remaining_tiles.extend(came_from.iter());
                    }
                }
            }
        }
        (path_tiles, total_steps)
    } else {
        (HashSet::new(), 0)
    }
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
