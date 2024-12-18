use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use crate::app::{class_string, DayOutput, Diagnostic, Tab};
use crate::grid::{Coord, Grid};

type Key = (Coord, Coord);
type TileData = (u64, HashSet<Key>);
type CandidateMap = HashMap<Key, TileData>;
type VisitedTile = (TileData, Direction);

#[derive(PartialEq, Clone, Debug)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
    Frontier,
    Split,
    Error,
}

#[derive(PartialEq, Clone, Debug)]
enum Tile {
    Wall,
    Empty,
    Visited(VisitedTile),
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
            if let Some((key, frontier_data)) = frontier.iter().next() {
                let key = key.clone();
                let frontier_data = frontier_data.clone();
                let frontier_cost = frontier_data.0;
                frontier.remove(&key);

                add_to_candidate_map(&mut explored, key, frontier_data);
                let (source, dir) = key;

                for (new_position, new_dir, new_cost) in [
                    (source.add(&dir), dir, frontier_cost + 1),
                    (source, dir.rotate_left(), frontier_cost + 1000),
                    (source, dir.rotate_right(), frontier_cost + 1000),
                ] {
                    let new_came_from = HashSet::from([key]);
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
                        add_to_candidate_map(&mut frontier, (new_position, new_dir), (new_cost, new_came_from));
                    }
                }
                if step < 200 {
                    add_tab(&input_grid, &mut tabs, &frontier, &explored, format!("S{}", step), source, dir);
                }
                step += 1;
            }
        }
        add_tab(&input_grid, &mut tabs, &frontier, &explored, format!("Final state, {} steps", step), Coord::new(0, 0), Coord::new(0, 0));

        if let Some((end_key, cost)) = get_end_tile(&mut explored, end) {
            silver = Some(cost);
            let path_tiles: HashSet<Coord> = follow_path(&explored, end_key);
            add_tab_gold(&input_grid, &mut tabs, &path_tiles, format!("Gold"));
            gold = path_tiles.len();
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

fn create_frontier(grid: &Grid<Tile>, start: Coord) -> CandidateMap {
    let start_tile = grid.get(start);
    if let Some(Tile::Empty) = start_tile {
        HashMap::from_iter([((start, Coord::new(1, 0)), (0, HashSet::new()))].into_iter())
    } else {
        HashMap::new()
    }
}


fn add_to_candidate_map(map: &mut CandidateMap, key: (Coord, Coord), new_data: TileData) {
    match map.entry(key) {
        Entry::Occupied(mut occupied) => {
            combine_tile_data(occupied.get_mut(), &new_data);
        }
        Entry::Vacant(vacant) => {
            vacant.insert(new_data);
        }
    }
}

// Returns true if first was used
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

fn combine_tile(first: VisitedTile, second: VisitedTile) -> VisitedTile {
    let ((first_cost, mut first_came_from), first_direction) = first;
    let ((second_cost, second_came_from), second_direction) = second;
    if second_cost < first_cost {
        ((second_cost, second_came_from), second_direction)
    } else if second_cost == first_cost {
        first_came_from.extend(second_came_from.iter());
        ((first_cost, first_came_from), Direction::Split)
    } else {
        ((first_cost, first_came_from), first_direction)
    }
}

fn get_end_tile(explored: &mut CandidateMap, end: Coord) -> Option<(Key, u64)> {
    let mut output = None;
    for dir in Coord::get_orthagonal_dirs().into_iter() {
        let key = (end, dir);
        if let Some((new_cost, _came_from)) = explored.get(&key) {
            match output {
                None => {
                    output = Some((key, *new_cost));
                }
                Some((_, cost_value)) => {
                    if *new_cost < cost_value {
                        output = Some((key, *new_cost));
                    }
                }
            }
        }
    }
    output
}

fn follow_path(map: &CandidateMap, end_key: Key) -> HashSet<Coord> {
    let mut remaining_tiles = HashSet::from([end_key]);
    let mut path_tiles = HashSet::new();
    while !remaining_tiles.is_empty() {
        if let Some(next_key) = remaining_tiles.iter().next() {
            let next_key = next_key.clone();
            remaining_tiles.take(&next_key);
            path_tiles.insert(next_key.0);
            if let Some((_cost, came_from)) = map.get(&next_key) {
                remaining_tiles.extend(came_from.iter());
            }
        }
    }
    path_tiles
}


//////////////////////////////////
// Visualization code
//////////////////////////////////
fn add_tab(input_grid: &Grid<Tile>, tabs: &mut Vec<Tab>, frontier: &CandidateMap, explored: &CandidateMap, title: String, source: Coord, dir: Coord) {
    let mut grids = HashMap::new();
    for dir in Coord::get_orthagonal_dirs() {
        grids.insert(dir, input_grid.clone());
    }
    // Apply the explored tiles to the grid
    for ((explored_pos, explored_dir), tile_data) in explored.iter() {
        grids.get_mut(explored_dir).map(|grid| {
            if let Some(tile_handle) = grid.get_mut(*explored_pos) {
                let tile_data = tile_data.clone();
                let direction = match explored_dir.deref() {
                    (1, 0) => Direction::Right,
                    (0, 1) => Direction::Down,
                    (-1, 0) => Direction::Left,
                    (0, -1) => Direction::Up,
                    _ => Direction::Error,
                };
                *tile_handle = Tile::Visited((tile_data, direction));
            }
        });
    }
    // Apply the frontier tiles to the grid
    for ((frontier_pos, frontier_dir), tile_data) in frontier.iter() {
        grids.get_mut(frontier_dir).map(|grid| {
            if let Some(tile_handle) = grid.get_mut(*frontier_pos) {
                let tile_data = tile_data.clone();
                *tile_handle = Tile::Visited((tile_data, Direction::Frontier));
            }
        });
    }
    // Merge the directional grids. Both on top of each other, and with a "mushed" one
    // The "mushed" one uses the smallest cost for each tile
    let mut grid = Grid::new();
    if let Some((first, rest)) = Coord::get_orthagonal_dirs().split_first() {
        if let Some(mut mushed_grid) = grids.remove(first) {
            let mut appended_grid = mushed_grid.clone();
            for coord in rest {
                if let Some(new_grid) = grids.remove(coord) {
                    mushed_grid.mush(&new_grid, |first, second| {
                        match (first, second) {
                            (Tile::Visited(first_visited), Tile::Visited(second_visited)) => {
                                let new_visited = combine_tile(first_visited.clone(), second_visited.clone());
                                *first_visited = new_visited;
                            }
                            (_, _) => {}
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
        grid: grid.to_tab_grid_title_class(|cell, x, y| {
            match cell.clone() {
                Tile::Wall => (String::new(), yew::Classes::new()),
                Tile::Empty => (String::new(), yew::Classes::new()),
                Tile::Visited((tile_data, _direction)) => {
                    let (cost, came_from) = tile_data;
                    let title = format!("{}, {:?}", cost, came_from);
                    let class = if x == 5 && y % 15 == 9 {
                        class_string("bg-slate-100 text-slate-900")
                    } else {
                        yew::Classes::new()
                    };
                    (title, class)
                }
                Tile::Path => (String::new(), yew::Classes::new()),
            }
        }),
    })
}

fn add_tab_gold(input_grid: &Grid<Tile>, tabs: &mut Vec<Tab>, path_tiles: &HashSet<Coord>, title: String) {
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
            Tile::Visited((_tile_data, direction)) => {
                match direction {
                    Direction::Up => f.write_str("^"),
                    Direction::Right => f.write_str(">"),
                    Direction::Down => f.write_str("v"),
                    Direction::Left => f.write_str("<"),
                    Direction::Frontier => f.write_str("o"),
                    Direction::Split => f.write_str("x"),
                    Direction::Error => f.write_str("!"),
                }
            }
            Tile::Path => f.write_str("o"),
        }
    }
}
