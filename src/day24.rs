use crate::app::{DayOutput, Diagnostic, Tab};
use crate::common;
use crate::common::combine_4;
use itertools::Itertools;
use regex::{Match, Regex};
use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
enum GateType {
    And,
    Xor,
    Or,
}

#[derive(Copy, Clone, Debug)]
enum Terminal<'a> {
    Bool(bool),
    Gate(&'a str, GateType, &'a str),
}

pub fn puzzle(input: &str) -> DayOutput {
    let mut errors: Vec<String> = Vec::new();
    let mut tabs: Vec<Tab> = Vec::new();
    let mut input_split = input.split("\n\n");
    let (input_terminals, input_logic) = (input_split.next(), input_split.next());
    let terminals = input_terminals
        .map(|input_states| {
            input_states
                .split("\n")
                .filter_map(|line| {
                    let mut split = line.split(": ");
                    if let (Some(terminal_name), Some(state)) = (split.next(), split.next()) {
                        state.parse::<u8>().ok().map(|state| (terminal_name, state))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or(Vec::new());

    let re_gates =
        Regex::new(r"(?P<first>\S*) (?P<gate>AND|XOR|OR) (?P<second>\S*) -> (?P<output>\S*)")
            .expect("Should compile");
    let logic_gates = input_logic
        .map(|input_logic| {
            re_gates
                .captures_iter(input_logic)
                .filter_map(|captures| {
                    combine_4(
                        captures.name("first").map(|m| m.as_str()),
                        parse_gate(captures.name("gate")),
                        captures.name("second").map(|m| m.as_str()),
                        captures.name("output").map(|m| m.as_str()),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or(Vec::new());

    let mut mapping = HashMap::new();
    for (terminal_name, value) in terminals.iter() {
        mapping.insert(*terminal_name, Terminal::Bool(*value != 0));
    }
    for (first, gate, second, output) in logic_gates.iter() {
        mapping.insert(*output, Terminal::Gate(first, *gate, second));
    }

    let re_input_x = Regex::new(r"x(?P<num>\d*)").expect("Should compile");
    let re_input_y = Regex::new(r"y(?P<num>\d*)").expect("Should compile");
    let re_output = Regex::new(r"z(?P<num>\d*)").expect("Should compile");
    let mut memoize = HashMap::<&str, bool>::new();
    let silver = calculate_output(&mut errors, &mapping, &mut memoize);


    let plantuml_lines = create_plantuml(
        &mut errors,
        &mapping,
        re_input_x,
        re_input_y,
        re_output,
        &mut memoize,
    );
    let mut tests = Vec::new();
    for i in 0..15 {
        for j in 0..15 {
            let output = calculate_output_given_inputs(&mut errors, &mapping, i, j);
            tests.push(format!("{} & {} = {}", i, j, output));
        }
    }

    tabs.push(Tab {
        title: "Tests".to_string(),
        strings: tests,
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Terminals".to_string(),
        strings: terminals.iter().map(|item| format!("{:?}", item)).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Gates".to_string(),
        strings: logic_gates
            .iter()
            .map(|item| format!("{:?}", item))
            .collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Memoize".to_string(),
        strings: memoize
            .iter()
            .sorted_by_key(|item| item.0)
            .map(|item| format!("({:?}, {:?})", *item.0, if *item.1 { 1 } else { 0 }))
            .collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Plantuml".to_string(),
        strings: plantuml_lines,
        grid: vec![],
    });

    DayOutput {
        silver_output: format!("{}", silver),
        gold_output: format!("{}", 0),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
    }
}

fn calculate_output_given_inputs(errors: &mut Vec<String>, mapping: &HashMap<&str, Terminal>, x: u64, y: u64) -> u64 {
    let mut mapping = mapping.clone();
    for bit in 0..64 {
        let x_key = format!("x{:02}", bit);
        let y_key = format!("y{:02}", bit);
        if let Some(x_terminal) = mapping.get_mut(&x_key as &str) {
            if let Terminal::Bool(value) = x_terminal {
                *value = x & (1 << bit) > 0;
            }
            if let Some(y_terminal) = mapping.get_mut(&y_key as &str) {
                if let Terminal::Bool(value) = y_terminal {
                    *value = y & (1 << bit) > 0;
                }
            } else {
                errors.push(format!("Had {} but not {}", x_key, y_key));
            }
        } else {
            // errors.push(format!("Setting x and y for {} bits", bit));
            break;
        }

    }
    let mut memoize = HashMap::new();
    calculate_output(errors, &mapping, &mut memoize)
}

fn calculate_output<'a, 'b: 'a>(
    errors: &mut Vec<String>,
    mapping: &'b HashMap<&'b str, Terminal>,
    mut memoize: &'a mut HashMap<&'b str, bool>,
) -> u64 {
    let re_output = Regex::new(r"z(?P<num>\d*)").expect("Should compile");
    let mut result = 0;
    for terminal in mapping.keys() {
        if let Some(captures) = re_output.captures(terminal) {
            if let Some(bit) = common::capture_parse::<u32>(&captures, "num") {
                let value = find_value(mapping, &mut memoize, terminal);
                match value {
                    Ok(value) => {
                        let value = if value { 1u64 } else { 0u64 };
                        result = result | (value << bit);
                    }
                    Err(error) => errors.push(error),
                }
            }
        }
    }
    result
}

fn create_plantuml(
    errors: &mut Vec<String>,
    mapping: &HashMap<&str, Terminal>,
    re_input_x: Regex,
    re_input_y: Regex,
    re_output: Regex,
    memoize: &mut HashMap<&str, bool>,
) -> Vec<String> {
    let mut plantuml_lines = Vec::new();
    plantuml_lines.push("@startuml".to_string());
    plantuml_lines.push("left to right direction".to_string());
    plantuml_lines.push("title Advent of Code 2024 day 24 diagram".to_string());
    for (terminal_name, value) in memoize.iter().sorted() {
        let group_name = if re_output.is_match(terminal_name) {
            "z"
        } else if re_input_x.is_match(terminal_name) {
            "x"
        } else if re_input_y.is_match(terminal_name) {
            "y"
        } else {
            "m"
        };
        if let Some(terminal) = mapping.get(terminal_name) {
            let kind = match terminal {
                Terminal::Bool(_) => "Input",
                Terminal::Gate(_, gate, _) => match gate {
                    GateType::And => "AND",
                    GateType::Xor => "XOR",
                    GateType::Or => "OR",
                },
            };
            plantuml_lines.push(format!("map {}.{} {{", group_name, terminal_name));
            plantuml_lines.push(format!("\t{} => {}", kind, if *value { 1 } else { 0 }));
            plantuml_lines.push("}".to_string());
        } else {
            errors.push(format!("Didn't find {} in mapping", terminal_name));
        }
    }
    for (terminal_name, terminal) in mapping.iter() {
        match terminal {
            Terminal::Bool(_) => {}
            Terminal::Gate(first, _, second) => {
                plantuml_lines.push(format!("{} --> {}", first, terminal_name));
                plantuml_lines.push(format!("{} --> {}", second, terminal_name));
            }
        }
    }
    plantuml_lines.push("@enduml".to_string());
    plantuml_lines
}

fn find_value<'a>(
    mapping: &'a HashMap<&'a str, Terminal>,
    memoize: &mut HashMap<&'a str, bool>,
    terminal_name: &'a str,
) -> Result<bool, String> {
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
