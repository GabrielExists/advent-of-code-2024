use std::iter::zip;
use gloo::net::websocket::State::Open;
use regex::{Regex};
use crate::app::{DayOutput, Diagnostic};

enum Operator {
    Add,
    Multiply,
    Concat,
}

pub fn puzzle(input: &str) -> DayOutput {
    let rows = input.split("\n").into_iter().filter_map(|line| {
        let mut split = line.split(": ");
        if let (Some(output), Some(rest)) = (split.next(), split.next()) {
            let values = rest.split(" ").into_iter().filter_map(|text| {
                text.parse::<u64>().ok()
            }).collect::<Vec<u64>>();
            if let Some((first, rest)) = values.split_first() {
                if let Some(target) = output.parse::<u64>().ok() {
                    Some((target, *first, rest.to_vec()))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }).collect::<Vec<(u64, u64, Vec<u64>)>>();

    let mut sum_of_silver = 0;
    let mut sum_of_gold = 0;
    let silver_operator_list = [Operator::Add, Operator::Multiply];
    let gold_operator_list = [Operator::Add, Operator::Multiply, Operator::Concat];
    for (target, first, rest) in rows {
        let passes = check(first, &rest, target, &silver_operator_list);
        if passes {
            sum_of_silver += target;
        }
        let passes = check(first, &rest, target, &gold_operator_list);
        if passes {
            sum_of_gold += target;
        }
    }

    DayOutput {
        silver_output: format!("{}", sum_of_silver),
        gold_output: format!("{}", sum_of_gold),
        diagnostic: Diagnostic::simple(format!("")),
    }
}

fn check(accumulator: u64, values: &[u64], target: u64, operator_list: &[Operator]) -> bool {
    match values.split_first() {
        None => {
            accumulator == target
        }
        Some((first, rest)) => {
            for operator in operator_list {
                if let Some(accumulator) = operator.apply(accumulator, *first) {
                    let pass = check(accumulator, rest, target, operator_list);
                    if pass {
                        return true;
                    }
                } else {
                    return false;
                }
            }
            false
        }
    }
}
impl Operator {
    fn apply(&self, one: u64, other: u64) -> Option<u64> {
        match self {
            Operator::Add => {Some(one + other)}
            Operator::Multiply => {Some(one * other)}
            Operator::Concat => {
                format!("{}{}", one.to_string(), other.to_string()).parse::<u64>().ok()
            }
        }
    }
}