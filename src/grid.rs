use std::fmt::{Debug, Display, Formatter};
use std::iter::zip;
use std::ops::Deref;
use yew::Classes;
use crate::app::GridCell;


#[derive(Clone, Debug)]
pub struct Grid<T> (pub Vec<Vec<T>>);

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub struct Coord(pub (i32, i32));

impl<T> Grid<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn new_repeat(x: usize, y: usize, item: T) -> Self
        where T: Clone {
        let grid = (0..y).map(|_| {
            (0..x).map(|_| {
                item.clone()
            }).collect()
        }).collect();
        Grid(grid)
    }

    pub fn from<F>(input: &str, mut cell_function: F) -> Self
        where F: FnMut(char) -> T {
        let grid = input.split("\n").map(|row| {
            row.chars().map(|character| {
                cell_function(character)
            }).collect::<Vec<T>>()
        }).filter(|a| !a.is_empty()).collect::<Vec<Vec<T>>>();
        Grid(grid)
    }

    pub fn from_filtered<F>(input: &str, mut cell_function: F) -> Self
        where F: FnMut(char) -> Option<T> {
        let grid = input.split("\n").map(|row| {
            row.chars().filter_map(|character| {
                cell_function(character)
            }).collect::<Vec<T>>()
        }).filter(|a| !a.is_empty()).collect::<Vec<Vec<T>>>();
        Grid(grid)
    }

    pub fn from_filtered_flatten<F>(input: &str, mut cell_function: F) -> Self
        where F: FnMut(char) -> Option<Vec<T>> {
        let grid = input.split("\n").map(|row| {
            row.chars().filter_map(|character| {
                cell_function(character)
            }).flatten().collect::<Vec<T>>()
        }).filter(|a| !a.is_empty()).collect::<Vec<Vec<T>>>();
        Grid(grid)
    }

    pub fn from_with_index<F>(input: &str, mut cell_function: F) -> Self
        where F: FnMut(char, i32, i32) -> T {
        let grid = input.split("\n").enumerate().map(|(y, row)| {
            row.chars().enumerate().map(|(x, character)| {
                cell_function(character, x as i32, y as i32)
            }).collect::<Vec<T>>()
        }).filter(|a| !a.is_empty()).collect::<Vec<Vec<T>>>();
        Grid(grid)
    }

    pub fn from_with_index_filtered<F>(input: &str, mut cell_function: F) -> Self
        where F: FnMut(char, i32, i32) -> Option<T> {
        let grid = input.split("\n").enumerate().map(|(y, row)| {
            row.chars().enumerate().filter_map(|(x, character)| {
                cell_function(character, x as i32, y as i32)
            }).collect::<Vec<T>>()
        }).filter(|a| !a.is_empty()).collect::<Vec<Vec<T>>>();

        Grid(grid)
    }

    pub fn add_row_from<U, F>(&mut self, input: &Vec<U>, cell_function: F)
        where F: Fn(&U) -> T {
        let new_row = input.iter().map(|item: &U| {
            cell_function(item)
        }).collect::<Vec<T>>();
        self.0.push(new_row);
    }

    pub(crate) fn find<F>(&self, predicate: F) -> Option<Coord>
        where F: Fn(&T) -> bool {
        self.0.iter().enumerate().fold(None, |output, (y, row)| {
            match output {
                Some(x) => Some(x),
                None => {
                    row.iter().enumerate().fold(None, |output, (x, tile)| {
                        match output {
                            Some(x) => Some(x),
                            None => {
                                if predicate(tile) {
                                    Some(Coord::new(x as i32, y as i32))
                                } else {
                                    None
                                }
                            }
                        }
                    })
                }
            }
        })
    }

    pub fn count<F>(&self, predicate: F) -> usize
        where F: Fn(&T) -> bool {
        self.0.iter().fold(0, |acc, row| {
            let subsum = row.iter().fold(0, |acc, cell| {
                if predicate(cell) {
                    acc + 1
                } else {
                    acc
                }
            });
            acc + subsum
        })
    }

    pub(crate) fn swap(&mut self, first: Coord, second: Coord) -> bool
        where T: Clone {
        if let Some(first_contents) = self.get(first) {
            let first_contents = first_contents.clone();
            if let Some(handle_second) = self.get_mut(second) {
                let second_contents = handle_second.clone();
                *handle_second = first_contents;
                if let Some(handle_first) = self.get_mut(first) {
                    *handle_first = second_contents;
                    return true;
                }
            }
        }
        false
    }
    pub(crate) fn append(&mut self, grid: Grid<T>) {
        self.0.extend(grid.0.into_iter());
    }

    pub(crate) fn mush<F>(&mut self, grid: &Grid<T>, merge_func: F)
        where F: Fn(&mut T, &T), T: Debug {
        log::info!("A {:?}", self);
        log::info!("B {:?}", grid);
        let _ = zip(self.0.iter_mut(), grid.0.iter()).map(|(self_row, grid_row)| {
            let _ = zip(self_row.iter_mut(), grid_row.iter()).map(|(self_item, other_item)| {
                merge_func(self_item, other_item);
            }).count();
        }).count();
    }

    pub fn to_tab_grid(&self) -> Vec<Vec<GridCell>>
        where T: Display {
        self.0.iter().map(|row| {
            row.iter().map(|cell| {
                GridCell {
                    text: cell.to_string(),
                    class: Classes::new(),
                    title: "".to_string(),
                }
            }).collect()
        }).collect()
    }
    pub fn to_tab_grid_class<F>(self, class_function: F) -> Vec<Vec<GridCell>>
        where T: Display, F: Fn(&T) -> Classes {
        self.0.into_iter().map(|row| {
            row.into_iter().map(|cell| {
                let class = class_function(&cell);
                GridCell {
                    text: cell.to_string(),
                    class,
                    title: "".to_string(),
                }
            }).collect()
        }).collect()
    }

    pub fn _to_tab_grid_title<F>(self, title_function: F) -> Vec<Vec<GridCell>>
        where T: Display, F: Fn(&T) -> String {
        self.0.into_iter().map(|row| {
            row.into_iter().map(|cell| {
                let title = title_function(&cell);
                GridCell {
                    text: cell.to_string(),
                    class: Classes::new(),
                    title,
                }
            }).collect()
        }).collect()
    }

    pub fn to_tab_grid_title_class<F>(self, title_function: F) -> Vec<Vec<GridCell>>
        where T: Display, F: Fn(&T, usize, usize) -> (String, Classes) {
        self.0.into_iter().enumerate().map(|(y, row)| {
            row.into_iter().enumerate().map(|(x, cell)| {
                let (title, class) = title_function(&cell, x, y);
                GridCell {
                    text: cell.to_string(),
                    class,
                    title,
                }
            }).collect()
        }).collect()
    }

    pub fn get_all_coords(&self) -> Vec<Coord> {
        self.0.iter().enumerate().map(|(y, row)| {
            row.iter().enumerate().map(move |(x, _)| {
                Coord::new(x as i32, y as i32)
            })
        }).flatten().collect::<Vec<_>>()
    }

    pub fn get(&self, coord: Coord) -> Option<&T> {
        if let Some((x, y)) = coord.into_usize() {
            self.0.get(y).map(|row| {
                row.get(x)
            }).unwrap_or(None)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, coord: Coord) -> Option<&mut T> {
        if let Some((x, y)) = coord.into_usize() {
            self.0.get_mut(y).map(|row| {
                row.get_mut(x)
            }).unwrap_or(None)
        } else {
            None
        }
    }
    pub(crate) fn map_grid<U, F>(&self, mut cell_function: F) -> Vec<Vec<U>>
        where F: FnMut(&T, usize, usize) -> U {
        self.0.iter().enumerate().map(|(y, row)| {
            row.iter().enumerate().map(|(x, cell)| {
                cell_function(cell, x, y)
            }).collect::<Vec<U>>()
        }).collect::<Vec<Vec<U>>>()
    }
}

impl Coord {
    pub fn new<T>(x: T, y: T) -> Self
        where T: Into<i32> {
        Coord((x.into(), y.into()))
    }
    pub fn add(&self, other: &Self) -> Self {
        Self((
            self.deref().0 + other.deref().0,
            self.deref().1 + other.deref().1,
        ))
    }
    pub fn subtract(&self, other: &Self) -> Self {
        Self((
            self.deref().0 - other.deref().0,
            self.deref().1 - other.deref().1,
        ))
    }
    pub fn multiply(&self, scalar: i32) -> Self {
        Self((
            self.deref().0 * scalar,
            self.deref().1 * scalar,
        ))
    }
    pub fn into_usize(self) -> Option<(usize, usize)> {
        let Coord((x, y)) = self;
        let (x, y): (Option<usize>, Option<usize>) = (x.try_into().ok(), y.try_into().ok());
        if let (Some(x), Some(y)) = (x, y) {
            Some((x, y))
        } else {
            None
        }
    }
    pub fn rotate_left(self) -> Self {
        Self((self.0.1, -self.0.0))
    }
    pub fn rotate_right(self) -> Self {
        Self((-self.0.1, self.0.0))
    }

    pub fn get_orthagonal_dirs() -> Vec<Self> {
        vec![
            Self::new(0, 1),
            Self::new(1, 0),
            Self::new(0, -1),
            Self::new(-1, 0),
        ]
    }
}

impl Deref for Coord {
    type Target = (i32, i32);

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({}, {})", self.deref().0, self.deref().1))
    }
}