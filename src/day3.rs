use regex::{Match, Regex};
use crate::app::{DayOutput, Diagnostic};


pub fn puzzle(input: &str) -> DayOutput {
    let re = Regex::new(r"(?P<mul>mul\((?P<first>\d*),(?P<second>\d*)\))|(?P<dont>don't\(\))|(?P<do>do\(\))").unwrap();

    let mut capture_summary = Vec::new();
    let actions = re.captures_iter(input)
        .filter_map(|captures| {
            capture_summary.push((
                captures.name("mul"),
                captures.name("dont"),
                captures.name("do"),
                captures.name("first"),
                captures.name("second"),
            ));
            if let Some(_) = captures.name("mul") {
                log::info!("mul");
                double_parse(captures.name("first"), captures.name("second")).map(|pair| Action::Multiply(pair))
            } else if let Some(_) = captures.name("dont") {
                log::info!("dont");
                Some(Action::Dont)
            } else if let Some(_) = captures.name("do") {
                log::info!("do");
                Some(Action::Do)
            } else {
                None
            }
        })
        .collect::<Vec<Action>>();

    let silver_sum = actions.iter()
        .filter_map(|action| {
            match action {
                Action::Multiply((first, second)) => Some(first * second),
                _ => None,
            }
        })
        .sum::<u64>();

    let (_, gold_sum) = actions.iter()
        .fold((true, 0), |(active, sum), action| {
            match action {
                Action::Multiply((first, second)) => {
                    let sum = if active {
                        sum + (*first * *second)
                    } else {
                        sum
                    };
                    (active, sum)
                }
                Action::Do => {
                    (true, sum)
                }
                Action::Dont => {
                    (false, sum)
                }
            }
        });
    DayOutput {
        silver_output: format!("{}", silver_sum),
        gold_output: format!("{}", gold_sum),
        // diagnostic: format!("Actions: {:?}, Captures: {:#?}", actions, capture_summary[0]),
        diagnostic: Diagnostic::simple(format!("Actions: {:?}", actions)),
    }
}

#[derive(Debug, Clone)]
enum Action {
    Multiply((u64, u64)),
    Do,
    Dont,
}

pub fn double_parse(first: Option<Match>, second: Option<Match>) -> Option<(u64, u64)> {
    match (
        first.map(|item| item.as_str().parse::<u64>()),
        second.map(|item| item.as_str().parse::<u64>())
    ) {
        (
            Some(Ok(first)),
            Some(Ok(second))
        ) => Some((first, second)),
        _ => None,
    }
}