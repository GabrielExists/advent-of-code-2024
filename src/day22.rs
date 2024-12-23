use crate::app::{DayOutput, Diagnostic, Tab};

pub fn puzzle(input: &str) -> DayOutput {
    let input_numbers = input.split("\n")
        .into_iter()
        .filter_map(|line| line.parse::<u64>().ok())
        .collect::<Vec<_>>();
    let errors: Vec<String> = Vec::new();
    let mut tabs: Vec<Tab> = Vec::new();

    let outputs = input_numbers.iter().map(|secret_number| {
        evolve_repeatedly(*secret_number, 2000)
    }).collect::<Vec<_>>();

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
        strings: outputs.iter().map(|item| format!("{}", item)).collect(),
        grid: vec![],
    });
    DayOutput {
        silver_output: format!("{}", outputs.into_iter().sum::<u64>()),
        gold_output: format!("{}", 0),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
    }
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

fn get_price(secret_number: u64) -> u64 {
    secret_number % 10
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
