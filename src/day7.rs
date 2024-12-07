use std::fmt::{Display, Formatter};
use crate::app::{DayOutput, Diagnostic, Tab};

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
    let mut silver_solutions = Vec::new();
    let mut gold_solutions  = Vec::new();
    for (target, first, rest) in rows {
        let solution = check(first, &rest, target, &silver_operator_list);
        if let Some(solution) = solution {
            add_solution(&mut silver_solutions, target, first, solution);
            sum_of_silver += target;
        } else {
            add_lack_of_solution(&mut silver_solutions, target, first, &rest);
        }
        let solution = check(first, &rest, target, &gold_operator_list);
        if let Some(solution) = solution {
            add_solution(&mut gold_solutions, target, first, solution);
            sum_of_gold += target;
        } else {
            add_lack_of_solution(&mut gold_solutions, target, first, &rest);
        }
    }

    let mut tabs = Vec::new();
    add_tab(&mut tabs, "Silver solutions", silver_solutions);
    add_tab(&mut tabs, "Gold solutions", gold_solutions);

    DayOutput {
        silver_output: format!("{}", sum_of_silver),
        gold_output: format!("{}", sum_of_gold),
        diagnostic: Diagnostic::with_tabs(tabs, format!("")),
    }
}


fn check<'a>(accumulator: u64, values: &[u64], target: u64, operator_list: &'a [Operator]) -> Option<Vec<(&'a Operator, u64)>> {
    match values.split_first() {
        None => {
            if accumulator == target {
                Some(Vec::new())
            } else {
                None
            }
        }
        Some((first, rest)) => {
            for operator in operator_list {
                if let Some(accumulator) = operator.apply(accumulator, *first) {
                    let solution = check(accumulator, rest, target, operator_list);
                    match solution {
                        Some(mut solution) => {
                            solution.push((operator, *first));
                            return Some(solution)
                        }
                        None => {}
                    }
                }
            }
            None
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
impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Add => f.write_str(" + "),
            Operator::Multiply => f.write_str(" * "),
            Operator::Concat => f.write_str(" || "),
        }
    }
}
fn add_solution(solutions: &mut Vec<String>, target: u64, first: u64, solution: Vec<(&Operator, u64)>) {
    let mut string = format!("{}: {}", target, first);
    for (operator, number) in solution {
        string = format!("{}{}{}", string, operator, number);
    }
    solutions.push(string);
}
fn add_lack_of_solution(solutions: &mut Vec<String>, target: u64, first: u64, solution: &[u64]) {
    let mut string = format!("! {}: {}", target, first);
    for number in solution {
        string = format!("{} {}", string, number);
    }
    solutions.push(string);
}

fn add_tab(tabs: &mut Vec<Tab>, title: &str, solutions: Vec<String>) {
    tabs.push(Tab {
        title: title.to_string(),
        strings: solutions,
        grid: vec![],
    })
}
