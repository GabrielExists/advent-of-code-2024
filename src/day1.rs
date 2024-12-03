use std::iter::zip;
use regex::{Regex};
use crate::app::DayOutput;

pub fn puzzle(input: &str) -> DayOutput {
    let re = Regex::new(r"^([0-9]*)   ([0-9]*)$").unwrap();
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

    let mut similarity_score = 0;
    for left in left_list.iter() {
        let occurrences = right_list.iter().filter(|right| **right == *left).count() as u64;
        similarity_score += left * occurrences;
    }

    DayOutput {
        silver_output: format!("{}", sum_of_distances),
        gold_output: format!("{}", similarity_score),
        diagnostic: format!("{:?}, {:?}", left_list, right_list),
    }
}