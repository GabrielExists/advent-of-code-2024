use crate::app::DayOutput;

#[derive(Clone, Debug)]
enum Letter {
    X,
    M,
    A,
    S,
}

pub fn puzzle(input: &str) -> DayOutput {
    let mut errors: Vec<String> = Vec::new();
    let grid = input.split("\n").map(|line| {
        log::info!("{:?}", line);
        line.chars().filter_map(|character| {
            log::info!("{:?}", character);
            match character {
                'X' => Some(Letter::X),
                'M' => Some(Letter::M),
                'A' => Some(Letter::A),
                'S' => Some(Letter::S),
                c => {
                    errors.push(format!("Found invalid character in input: {}", c));
                    None
                }
            }
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    let mut num_found = 0;
    for y in 0..grid.len() {
        if let Some(row) = grid.get(y) {
            for x in 0..(*row).len() {
                let coord = (x as i32, y as i32);
                if let Some(Letter::X) = find_coord(&grid, coord) {
                    for dir in [
                        (-1, -1),
                        (-1, 0),
                        (-1, 1),
                        (0, -1),
                        (0, 1),
                        (1, -1),
                        (1, 0),
                        (1, 1),
                    ] {
                        let next_coord = add_coord(coord, dir);
                        if let Some(Letter::M) = find_coord(&grid, next_coord) {
                            let next_coord = add_coord(next_coord, dir);
                            if let Some(Letter::A) = find_coord(&grid, next_coord) {
                                let next_coord = add_coord(next_coord, dir);
                                if let Some(Letter::S) = find_coord(&grid, next_coord) {
                                    num_found += 1;
                                    errors.push(format!("Found in direction {:?} starting at {:?} ending at {:?}", dir, coord, next_coord));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut num_found_gold = 0;
    for y in 0..grid.len() {
        if let Some(row) = grid.get(y) {
            for x in 0..(*row).len() {
                let coord = (x as i32, y as i32);
                if let Some(Letter::A) = find_coord(&grid, coord) {
                    if let Some(down_right) = find_coord(&grid, add_coord(coord, (1, 1))) {
                        match down_right {
                            Letter::M => {
                                if let Some(Letter::S) = find_coord(&grid, add_coord(coord, (-1, -1))) {
                                } else {
                                    continue
                                }
                            }
                            Letter::S => {
                                if let Some(Letter::M) = find_coord(&grid, add_coord(coord, (-1, -1))) {
                                } else {
                                    continue
                                }
                            }
                            _ => {
                                continue
                            }
                        }
                    } else {
                        continue
                    }
                    if let Some(down_right) = find_coord(&grid, add_coord(coord, (1, -1))) {
                        match down_right {
                            Letter::M => {
                                if let Some(Letter::S) = find_coord(&grid, add_coord(coord, (-1, 1))) {
                                } else {
                                    continue
                                }
                            }
                            Letter::S => {
                                if let Some(Letter::M) = find_coord(&grid, add_coord(coord, (-1, 1))) {
                                } else {
                                    continue
                                }
                            }
                            _ => {
                                continue
                            }
                        }
                    } else {
                        continue
                    }
                    num_found_gold += 1;
                }
            }
        }
    }

    DayOutput {
        silver_output: format!("{}", num_found),
        gold_output: format!("{}", num_found_gold),
        diagnostic: format!("errors: {:?}, grid: {:?}", errors, grid.get(1)),
    }
}

fn find_coord(grid: &Vec<Vec<Letter>>, coord: (i32, i32)) -> Option<Letter> {
    let (x, y): (Option<usize>, Option<usize>) = (coord.0.try_into().ok(), coord.1.try_into().ok());
    if let (Some(x), Some(y)) = (x, y) {
        grid.get(y).map(|row| {
            row.get(x).map(|letter| letter.clone())
        }).unwrap_or(None)
    } else {
        None
    }
}

fn add_coord(first: (i32, i32), second: (i32, i32)) -> (i32, i32) {
    (first.0 + second.0, first.1 + second.1)
}