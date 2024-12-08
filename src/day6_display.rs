use crate::day6_old::*;
use std::fmt::{Display, Formatter};
use std::convert::TryInto;
use crate::app::*;

#[derive(Clone, Debug)]
pub enum OutputLetter {
    Dot,
    Hash,
    Guard,
    Walked,
    Checked,
    Obstacle,
    CheckingStartLocation,
    CheckingObstacle,
    Counter,
}

impl Display for OutputLetter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputLetter::Dot => f.write_str("."),
            OutputLetter::Hash => f.write_str("#"),
            OutputLetter::Guard => f.write_str("^"),
            OutputLetter::Walked => f.write_str("X"),
            OutputLetter::Checked => f.write_str("/"),
            OutputLetter::Obstacle => f.write_str("O"),
            OutputLetter::CheckingStartLocation => f.write_str("C"),
            OutputLetter::CheckingObstacle => f.write_str("Ø"),
            OutputLetter::Counter => f.write_str("L"),
        }
    }
}

impl From<&Letter> for OutputLetter {
    fn from(value: &Letter) -> Self {
        match value {
            Letter::Dot => Self::Dot,
            Letter::Hash => Self::Hash,
            Letter::Guard => Self::Guard,
        }
    }
}

impl OutputLetter {
    fn to_cell(&self) -> GridCell {
        let class = match self {
            OutputLetter::Dot => "",
            OutputLetter::Hash => "",
            _ => "",
        };
        GridCell {
            text: self.to_string(),
            class: class_string(&class),
        }
    }
}

pub fn apply_locations_to_output_grid<T: IntoIterator<Item=Coord>>(output_grid: &mut Vec<Vec<OutputLetter>>, locations: T, new_letter: OutputLetter) {
    for coord in locations.into_iter() {
        let letter = find_output_coord_mut(output_grid, coord);
        if let Some(letter) = letter {
            *letter = new_letter.clone();
        }
    }
}

pub fn cells_from_output_grid(grid: &Vec<Vec<OutputLetter>>) -> Vec<Vec<GridCell>> {
    grid.iter().map(|row| {
        row.iter().map(|letter| {
            letter.to_cell()
        }).collect::<Vec<GridCell>>()
    }).collect::<Vec<Vec<GridCell>>>()
}

pub fn output_grid_from_grid(grid: &Vec<Vec<Letter>>) -> Vec<Vec<OutputLetter>> {
    grid.iter().map(|row| {
        row.iter().map(|letter| {
            letter.into()
        }).collect::<Vec<OutputLetter>>()
    }).collect::<Vec<Vec<OutputLetter>>>()
}

fn find_output_coord_mut(grid: &mut Vec<Vec<OutputLetter>>, coord: (i32, i32)) -> Option<&mut OutputLetter> {
    let (x, y): (Option<usize>, Option<usize>) = (coord.0.try_into().ok(), coord.1.try_into().ok());
    if let (Some(x), Some(y)) = (x, y) {
        grid.get_mut(y).map(|row| {
            row.get_mut(x)
        }).unwrap_or(None)
    } else {
        None
    }
}
