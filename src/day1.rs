use std::iter::zip;
use regex::{Captures, Regex};

pub fn puzzle(input: &str) -> String {
    let re = Regex::new(r"^([0-9]*)   ([0-9]*)$").unwrap();
    let mut output = String::new();
    let pairs = input.split("\n")
        .filter_map(|line| {
            re.captures(line)
                .and_then(|captures| {
                    if let (Some(left_match), Some(right_match)) = (
                        captures.get(1),
                        captures.get(2)
                    ) {
                        if let (Ok(left), Ok(right)) = (
                            left_match.as_str().parse::<u64>(),
                            right_match.as_str().parse::<u64>()
                        ) {
                            Some((left, right))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
        })
        .collect::<Vec<_>>();

    let mut left_list = Vec::new();
    let mut right_list = Vec::new();
    for pair in pairs {
        left_list.push(pair.0);
        right_list.push(pair.1);
    }
    left_list.sort();
    right_list.sort();

    let mut sum_of_distances = 0;
    for (left, right) in zip(left_list.iter(), right_list.iter()) {
        if left > right {
            sum_of_distances += left - right;
        } else {
            sum_of_distances += right - left;
        }
    }

    format!("{}", sum_of_distances)
}