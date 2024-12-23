use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use crate::app::{DayOutput, Diagnostic, Tab};

pub fn puzzle(input: &str) -> DayOutput {
    let input_numbers = input.split("\n")
        .into_iter()
        .filter_map(|line| line.parse::<u64>().ok())
        .collect::<Vec<_>>();
    let errors: Vec<String> = Vec::new();
    let mut tabs: Vec<Tab> = Vec::new();

    let outputs_silver = input_numbers.iter().map(|secret_number| {
        evolve_repeatedly(*secret_number, 2000)
    }).collect::<Vec<_>>();

    let sequence_to_profit_map: HashMap<Vec<i8>, u64> = generate_profit_map(&input_numbers);
    let best_sequence = sequence_to_profit_map.iter().max_by_key(|(_sequence, profit)| **profit);

    tabs.push(Tab {
        title: "Inputs".to_string(),
        strings: input_numbers.iter().map(|item| format!("{}", item)).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Stepwise 123".to_string(),
        strings: evolve_stepwise(123, 10).iter().map(|item| format!("{}", item)).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Outputs".to_string(),
        strings: outputs_silver.iter().map(|item| format!("{}", item)).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Best sequence".to_string(),
        strings: vec![best_sequence.map(|(seq, profit)|format!("Sequence: {:?}, profit: {}", seq, profit)).unwrap_or(String::new())],
        grid: vec![],
    });
    DayOutput {
        silver_output: format!("{}", outputs_silver.into_iter().sum::<u64>()),
        gold_output: format!("{}", best_sequence.map(|(seq, price)| *price).unwrap_or(0)),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
    }
}

fn generate_profit_map(inputs: &Vec<u64>) -> HashMap<Vec<i8>, u64> {
    let mut global_profit_map = HashMap::new();
    for secret_number in inputs {
        let mut secret_number = *secret_number;
        let mut profit_map: HashMap<Vec<i8>, u64> = HashMap::new();
        let mut sequence = Vec::new();
        let mut last_price = None;
        for _ in 0..2000 {
            let price = get_price(secret_number);
            if let Some(last_price_value) = last_price {
                let price_change = get_price_change(last_price_value, price);
                sequence.push(price_change);
            }
            while sequence.len() > 4 {
                sequence.remove(0);
            }
            if sequence.len() == 4 {
                if let Vacant(entry) = profit_map.entry(sequence.clone()) {
                    entry.insert(price as u64);
                }
            }
            last_price = Some(price);
            secret_number = evolve(secret_number);
        }
        for (sequence, price) in profit_map.into_iter() {
            let entry = global_profit_map.entry(sequence).or_insert(0);
            *entry += price;
        }
    }
    global_profit_map
}


fn evolve_stepwise(mut secret_number: u64, steps: u64) -> Vec<u64> {
    let mut output = vec![secret_number];
    for _ in 0..steps {
        secret_number = evolve(secret_number);
        output.push(secret_number);
    }
    output
}

fn evolve_repeatedly(mut secret_number: u64, steps: u64) -> u64 {
    for _ in 0..steps {
        secret_number = evolve(secret_number);
    }
    secret_number
}

fn get_price(secret_number: u64) -> u8 {
    (secret_number % 10) as u8
}
fn get_price_change(last_price: u8, price: u8) -> i8 {
    last_price as i8 - price as i8
}

fn evolve(secret_number: u64) -> u64 {
    let secret_number = prune(mix(secret_number, secret_number * 64));
    let secret_number = prune(mix(secret_number, secret_number / 32));
    let secret_number = prune(mix(secret_number, secret_number * 2048));
    secret_number
}

fn mix(secret_number: u64, value: u64) -> u64 {
    secret_number ^ value
}

fn prune(secret_number: u64) -> u64 {
    secret_number % 16777216
}
