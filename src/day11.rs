use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use js_sys::JSON::stringify;
use crate::app::{DayOutput, Diagnostic, Tab};

pub fn puzzle(input: &str) -> DayOutput {
    let input_values = input.split(" ").into_iter().filter_map(|text| {
        text.parse::<u64>().ok()
    }).collect::<Vec<u64>>();

    let num_evolutions = 25;
    let mut evolutions = vec![format!("{:?}", input_values)];

    let mut values = input_values.clone();
    for _ in 0..num_evolutions {
        values = values.into_iter().map(|value| {
            if value == 0 {
                vec![1]
            } else {
                let string = format!("{}", value);
                if string.len() % 2 == 0 {
                    let split = string.split_at(string.len() / 2);
                    vec![split.0, split.1].into_iter().filter_map(|string| string.parse::<u64>().ok()).collect::<Vec<u64>>()
                } else {
                    vec![value * 2024]
                }
            }
        }).flatten().collect::<Vec<u64>>();

        evolutions.push(format!("{:?}", values));
    }

    let mut tabs = Vec::new();
    tabs.push(Tab {
        title: "Tab".to_string(),
        strings: evolutions,
        grid: vec![],
    });

    DayOutput {
        silver_output: format!("{}", values.len()),
        gold_output: format!("{}", 0),
        diagnostic: Diagnostic::with_tabs(tabs, format!("")),
    }
}

