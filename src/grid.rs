use std::fmt::Display;
use std::ops::Deref;
use crate::app::GridCell;


#[derive(Clone, Debug)]
pub struct Grid<T> (Vec<Vec<T>>);

impl<T> Grid<T> {}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub struct Coord((i32, i32));

impl<T> Grid<T> {
    pub fn new() -> Self {
        Self(Vec::new())
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

    pub fn to_tab_grid(self) -> Vec<Vec<GridCell>>
        where T: Display {
        self.0.into_iter().map(|row| {
            row.into_iter().map(|cell| {
                GridCell {
                    text: cell.to_string(),
                    class: Default::default(),
                }
            }).collect()
        }).collect()
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
    pub(crate) fn map_grid<U, F>(&self, cell_function: F) -> Vec<Vec<U>>
        where F: Fn(&T, usize, usize) -> U {
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
}

impl Deref for Coord {
    type Target = (i32, i32);

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}