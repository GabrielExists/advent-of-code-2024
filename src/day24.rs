use std::collections::HashMap;
use regex::{Match, Regex};
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::common;
use crate::common::combine_4;

#[derive(Copy, Clone, Debug)]
enum GateType {
    And,
    Xor,
    Or,
}

enum Terminal<'a> {
    Bool(bool),
    Gate(&'a str, GateType, &'a str),
}

pub fn puzzle(input: &str) -> DayOutput {
    let mut errors: Vec<String> = Vec::new();
    let mut tabs: Vec<Tab> = Vec::new();
    let mut input_split = input.split("\n\n");
    let (input_terminals, input_logic) = (input_split.next(), input_split.next());
    let terminals = input_terminals.map(|input_states| {
        input_states.split("\n").filter_map(|line| {
            let mut split = line.split(": ");
            if let (Some(terminal_name), Some(state)) = (split.next(), split.next()) {
                state.parse::<u8>().ok().map(|state| (terminal_name, state))
            } else {
                None
            }
        }).collect::<Vec<_>>()
    }).unwrap_or(Vec::new());

    let re_gates = Regex::new(r"(?P<first>\S*) (?P<gate>AND|XOR|OR) (?P<second>\S*) -> (?P<output>\S*)").expect("Should compile");
    let logic_gates = input_logic.map(|input_logic| {
        re_gates.captures_iter(input_logic).filter_map(|captures| {
            combine_4(
                captures.name("first").map(|m| m.as_str()),
                parse_gate(captures.name("gate")),
                captures.name("second").map(|m| m.as_str()),
                captures.name("output").map(|m| m.as_str()),
            )
        }).collect::<Vec<_>>()
    }).unwrap_or(Vec::new());

    let mut mapping = HashMap::new();
    for (terminal_name, value) in terminals.iter() {
        mapping.insert(*terminal_name, Terminal::Bool(*value != 0));
    }
    for (first, gate, second, output) in logic_gates.iter() {
        mapping.insert(*output, Terminal::Gate(first, *gate, second));
    }

    let re_output = Regex::new(r"z(?P<num>\d*)").expect("Should compile");
    let mut memoize = HashMap::<&str, bool>::new();
    let mut silver: u64 = 0;
    for terminal in mapping.keys() {
        if let Some(captures) = re_output.captures(terminal) {
            if let Some(bit) = common::capture_parse::<u32>(&captures, "num") {
                let value = find_value(&mapping, &mut memoize, terminal);
                match value {
                    Ok(value) => {
                        let value = if value { 1u64 } else { 0u64 };
                        silver = silver | (value << bit);
                    }
                    Err(error) => errors.push(error),
                }
            }
        }
    }

    tabs.push(Tab {
        title: "Terminals".to_string(),
        strings: terminals.iter().map(|item| format!("{:?}", item)).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Gates".to_string(),
        strings: logic_gates.iter().map(|item| format!("{:?}", item)).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Memoize".to_string(),
        strings: memoize.iter().map(|item| format!("{:?}", item)).collect(),
        grid: vec![],
    });

    DayOutput {
        silver_output: format!("{}", silver),
        gold_output: format!("{}", 0),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
    }
}

fn find_value<'a>(mapping: &'a HashMap<&'a str, Terminal>, memoize: &mut HashMap<&'a str, bool>, terminal_name: &'a str) -> Result<bool, String> {
    if let Some(value) = memoize.get(terminal_name) {
        return Ok(*value);
    }
    if let Some(terminal) = mapping.get(terminal_name) {
        match terminal {
            Terminal::Bool(value) => {
                memoize.insert(terminal_name, *value);
                Ok(*value)
            }
            Terminal::Gate(first_name, gate, second_name) => {
                let first = find_value(mapping, memoize, first_name)?;
                let second = find_value(mapping, memoize, second_name)?;
                let output = gate.apply(first, second);
                memoize.insert(terminal_name, output);
                Ok(output)
            }
        }
    } else {
        Err(format!("Couldn't find terminal {}", terminal_name))
    }
}

fn parse_gate(value: Option<Match>) -> Option<GateType> {
    if let Some(value) = value {
        match value.as_str() {
            "AND" => Some(GateType::And),
            "XOR" => Some(GateType::Xor),
            "OR" => Some(GateType::Or),
            _ => None,
        }
    } else {
        None
    }
}

impl GateType {
    fn apply(&self, first: bool, second: bool) -> bool {
        match self {
            GateType::And => first && second,
            GateType::Xor => first ^ second,
            GateType::Or => first || second,
        }
    }
}