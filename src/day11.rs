use std::collections::HashMap;
use crate::app::{DayOutput, Diagnostic, Tab};

pub fn puzzle(input: &str) -> DayOutput {
    let input_values = input.split(" ").into_iter().filter_map(|text| {
        text.parse::<u128>().ok()
    }).collect::<Vec<u128>>();

    let mut evolutions_silver = vec![];
    let mut evolutions_gold = vec![];

    let (sum_silver, errors) = apply_evolutions(&input_values, 25, &mut evolutions_silver);
    let (sum_gold, errors_gold  ) = apply_evolutions(&input_values, 75, &mut evolutions_gold);

    let mut tabs = Vec::new();
    tabs.push(Tab { // Apparently 165674 is wrong
        title: "Silver".to_string(),
        strings: evolutions_silver,
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Gold".to_string(),
        strings: evolutions_gold,
        grid: vec![],
    });

    DayOutput {
        silver_output: format!("{}", sum_silver),
        gold_output: format!("{}", sum_gold),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?} {:?}", errors, errors_gold)),
    }
}

fn apply_evolutions(input_values: &Vec<u128>, num_evolutions: usize, evolutions: &mut Vec<String>) -> (u128, Vec<String>) {
    *evolutions = vec![format!("{:?}", input_values)];
    let errors = Vec::new();
    let mut values: HashMap<u128, u128> = HashMap::new();
    for input_value in input_values {
        let entry = values.entry(*input_value).or_insert(0);
        *entry += 1;
    }
    for _ in 0..num_evolutions {
        let mut new_values = HashMap::new();
        for (value, num_values) in values.into_iter() {
            if value == 0 {
                add_value(&mut new_values, 1, num_values);
            } else {
                let string = format!("{}", value);
                let len = string.len();
                if len % 2 == 0 {
                    let half_power = 10u128.pow(len as u32 / 2);
                    let new_value = value / half_power;
                    add_value(&mut new_values, new_value, num_values);
                    let new_value = value % half_power;
                    add_value(&mut new_values, new_value, num_values);
                } else {
                    add_value(&mut new_values, value * 2024, num_values);
                }
            }
        }
        values = new_values;

        evolutions.push(format!("{:?}", values));
    }
    let sum =values.into_iter().map(|(_value, num)|num).sum();
    (sum, errors)
}

fn add_value(new_values: &mut HashMap<u128, u128>, new_value: u128, num_values: u128) {
    let entry = new_values.entry(new_value).or_insert(0);
    *entry += num_values;
}

