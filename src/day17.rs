use std::collections::{HashMap, HashSet};
use std::mem::offset_of;
use regex::Regex;
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::common::capture_parse;

#[derive(PartialEq, Clone, Debug)]
enum Instruction {
    ADV(Combo),
    BXL(u8),
    BST(Combo),
    JNZ(u8),
    BXC,
    OUT(Combo),
    BDV(Combo),
    CDV(Combo),
}

#[derive(PartialEq, Clone, Debug)]
enum Combo {
    Zero,
    One,
    Two,
    Three,
    A,
    B,
    C,
}

#[derive(PartialEq, Clone, Debug)]
struct State {
    instruction: usize,
    a: u64,
    b: u64,
    c: u64,
    output: Vec<u8>,
}


pub fn puzzle(input: &str) -> DayOutput {
    let re = Regex::new(r"Register A: (?P<a>\d+)\nRegister B: (?P<b>\d+)\nRegister C: (?P<c>\d+)\n\nProgram: (?P<program>\d(,\d)*)").unwrap();
    let mut errors = Vec::new();
    let mut output_silver: Option<Vec<u8>> = None;
    let mut output_gold = None;
    let mut diagnostic_gold = Vec::new();
    let mut diagnostic_stepped = Vec::new();
    if let Some(captures) = re.captures(input) {
        let a = capture_parse(&captures, "a");
        let b = capture_parse(&captures, "b");
        let c = capture_parse(&captures, "c");
        let program: Option<Vec<u8>> = captures.name("program").map(|m| {
            m.as_str().split(",").filter_map(|item| {
                item.parse::<u8>().ok()
            }).collect::<Vec<u8>>()
        });
        if let (Some(a), Some(b), Some(c), Some(program)) = (a, b, c, program) {
            let input_state = State {
                instruction: 0,
                a,
                b,
                c,
                output: vec![],
            };
            let silver_state = run_program(&program, input_state.clone(), 1000);
            output_silver = Some(silver_state.output.clone());

            let mut subprograms = Vec::new();
            for i in (0..(program.len())).rev() {
                subprograms.push(program[i..].to_vec());
            }
            errors.push(format!("{:?}", subprograms));
            match gold_find_next(&program, input_state.clone(), &subprograms, 0, &mut diagnostic_gold) {
                Ok(successes) => {
                    errors.push(format!("Gold solutions: {:?}", successes));
                    output_gold = successes.into_iter().min();
                }
                Err(longest_failure) => {
                    errors.push(format!("Couldn't find subprogram {:?}", longest_failure));
                }
            }

            diagnostic_stepped = run_program_step(&program, input_state.clone(), 1000);
        }
    }
    let mut tabs = Vec::new();
    let mut buckets = HashMap::new();
    for (_i, list) in diagnostic_gold.iter() {
        let bucket = buckets.entry(list.output.len()).or_insert(0);
        *bucket += 1;
    }
    tabs.push(Tab {
        title: "Stepped".to_string(),
        strings: diagnostic_stepped.into_iter().map(|a| format!("{:?}", a)).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Progression".to_string(),
        strings: diagnostic_gold.into_iter().take(3000).map(|a| format!("{:?}", a)).collect(),
        grid: vec![],
    });

    let formatted_output = output_silver.map(|list| list.into_iter().map(|number| number.to_string()).collect::<Vec<_>>().join(","));
    DayOutput {
        silver_output: format!("{}", formatted_output.unwrap_or(String::new())),
        gold_output: format!("{}", output_gold.map(|set| format!("{:?}", set)).unwrap_or(String::new())),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
    }
}

fn gold_find_next(program: &[u8], input_state: State, remaining_subprograms: &[Vec<u8>], a: u64, attempts: &mut Vec<(u64, State)>) -> Result<HashSet<u64>, Vec<u8>> {
    match remaining_subprograms.split_first() {
        None => {
            Ok(HashSet::from_iter([a]))
        }
        Some((subprogram, remaining_subprograms)) => {
            let first_attempt = a * 8;
            let last_attempt = first_attempt + 7;
            let mut successes: HashSet<u64> = HashSet::new();
            let mut longest_failure: Option<Vec<u8>> = None;
            for attempted_a in first_attempt..=last_attempt {
                let state = State {
                    a: attempted_a,
                    ..input_state.clone()
                };
                let output_state = run_program(&program, state, 1000);
                attempts.push((a, output_state.clone()));
                if output_state.output == *subprogram {
                    match gold_find_next(program, input_state.clone(), remaining_subprograms, attempted_a, attempts) {
                        Ok(success) => {
                            successes.extend(success.into_iter());
                        }
                        Err(failure) => {
                            match &longest_failure {
                                None => {}
                                Some(current_longest) => {
                                    if failure.len() > current_longest.len() {
                                        longest_failure = Some(failure);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !successes.is_empty() {
                Ok(successes)
            } else {
                if let Some(longest_failure) = longest_failure {
                    Err(longest_failure)
                } else {
                    Err(subprogram.clone())
                }
            }
        }
    }
}

fn run_program(instructions: &[u8], mut state: State, max_commands: usize) -> State {
    let mut num_commands = 0;
    loop {
        if num_commands > max_commands {
            break state;
        }
        if let (Some(opcode), Some(operand)) = (instructions.get(state.instruction), instructions.get(state.instruction + 1)) {
            if let Some(inst) = decode_instruction(*opcode, *operand) {
                state = apply_instruction(inst, state)
            } else {
                break state;
            }
        } else {
            break state;
        }
        num_commands += 1;
    }
}

fn run_program_step(instructions: &[u8], mut state: State, max_commands: usize) -> Vec<(Instruction, State)> {
    let mut num_commands = 0;
    let mut states = Vec::new();
    loop {
        if num_commands > max_commands {
            break;
        }
        if let (Some(opcode), Some(operand)) = (instructions.get(state.instruction), instructions.get(state.instruction + 1)) {
            if let Some(inst) = decode_instruction(*opcode, *operand) {
                state = apply_instruction(inst.clone(), state);
                states.push((inst, state.clone()));
            } else {
                break;
            }
        } else {
            break;
        }
        num_commands += 1;
    };
    states
}

// (BST(A), State { instruction: 2, a: 2203, b: 3, c: 0, output: [] })     // b = a % 8
// (BXL(5), State { instruction: 4, a: 2203, b: 6, c: 0, output: [] })     // b = b xor 5
// (CDV(B), State { instruction: 6, a: 2203, b: 6, c: 34, output: [] })    // c = a / 2**b
// (ADV(Three), State { instruction: 8, a: 275, b: 6, c: 34, output: [] }) // a = a / 8
// (BXC, State { instruction: 10, a: 275, b: 36, c: 34, output: [] })      // b = b xor c
// (BXL(6), State { instruction: 12, a: 275, b: 34, c: 34, output: [] })   // b = b xor 6
// (OUT(B), State { instruction: 14, a: 275, b: 34, c: 34, output: [2] })  // output = b % 8
// (JNZ(0), State { instruction: 0, a: 275, b: 34, c: 34, output: [2] })   // repeat

// (OUT(B), State { instruction: 14, a: 275, b: 34, c: 34, output: [2] })
// (BXL(6), State { instruction: 12, a: 275, b: 34, c: 34, output: [] })   // b = output + n * 8
// (BXC, State { instruction: 10, a: 275, b: 36, c: 34, output: [] })      // b = b xor 6
// (ADV(Three), State { instruction: 8, a: 275, b: 6, c: 34, output: [] }) // b = b xor c
// (CDV(B), State { instruction: 6, a: 2203, b: 6, c: 34, output: [] })    // a = a * 8 + m
// (BXL(5), State { instruction: 4, a: 2203, b: 6, c: 0, output: [] })     // c = a / 2**b
// (BST(A), State { instruction: 2, a: 2203, b: 3, c: 0, output: [] })     // b = b xor 5
// (INIT, State { instruction: 0, a: 2203, b: 0, c: 0, output: [] })       // b = a % 8


// (JNZ(0), State { instruction: 0, a: 4, b: 1, c: 0, output: [2, 4, 1] })
// (BST(A), State { instruction: 2, a: 4, b: 4, c: 0, output: [2, 4, 1] })     // b = a % 8
// (BXL(5), State { instruction: 4, a: 4, b: 1, c: 0, output: [2, 4, 1] })     // b = b xor 5
// (CDV(B), State { instruction: 6, a: 4, b: 1, c: 2, output: [2, 4, 1] })     // c = a / 2**b
// (ADV(Three), State { instruction: 8, a: 0, b: 1, c: 2, output: [2, 4, 1] }) // a = a / 8
// (BXC, State { instruction: 10, a: 0, b: 3, c: 2, output: [2, 4, 1] })       // b = b xor c
// (BXL(6), State { instruction: 12, a: 0, b: 5, c: 2, output: [2, 4, 1] })    // b = b xor 6
// (OUT(B), State { instruction: 14, a: 0, b: 5, c: 2, output: [2, 4, 1, 5] }) // output = b % 8
// (JNZ(0), State { instruction: 16, a: 0, b: 5, c: 2, output: [2, 4, 1, 5] })

// Reverse
// (JNZ(0), State { instruction: 16, a: 0, b: 5, c: 2, output: [2, 4, 1, 5] })
// (OUT(B), State { instruction: 14, a: 0, b: 5, c: 2, output: [2, 4, 1, 5] })
// (BXL(6), State { instruction: 12, a: 0, b: 5, c: 2, output: [2, 4, 1] })    // b = output + n * 8
// (BXC, State { instruction: 10, a: 0, b: 3, c: 2, output: [2, 4, 1] })       // b = b xor 6
// (ADV(Three), State { instruction: 8, a: 0, b: 1, c: 2, output: [2, 4, 1] }) // b = b xor c
// (CDV(B), State { instruction: 6, a: 4, b: 1, c: 2, output: [2, 4, 1] })     // a = a * 8 + m
// (BXL(5), State { instruction: 4, a: 4, b: 1, c: 0, output: [2, 4, 1] })     // c = a * 2**b + m
// (BST(A), State { instruction: 2, a: 4, b: 4, c: 0, output: [2, 4, 1] })     // b = b xor 5
// (JNZ(0), State { instruction: 0, a: 4, b: 1, c: 0, output: [2, 4, 1] })     // b = a * 8 + m

// 2,4,1,5,7,5,0,3,4,1,1,6,5,5,3,0
// 2,1,7,0,4,1,5,3


// (BST(A), State { instruction: 2, a: 47719761, b: 1, c: 0, output: [] })
// (BXL(5), State { instruction: 4, a: 47719761, b: 4, c: 0, output: [] })
// (CDV(B), State { instruction: 6, a: 47719761, b: 4, c: 2982485, output: [] })
// (ADV(Three), State { instruction: 8, a: 5964970, b: 4, c: 2982485, output: [] })
// (BXC, State { instruction: 10, a: 5964970, b: 2982481, c: 2982485, output: [] })
// (BXL(6), State { instruction: 12, a: 5964970, b: 2982487, c: 2982485, output: [] })
// (OUT(B), State { instruction: 14, a: 5964970, b: 2982487, c: 2982485, output: [7] })
// (JNZ(0), State { instruction: 0, a: 5964970, b: 2982487, c: 2982485, output: [7] })
// (BST(A), State { instruction: 2, a: 5964970, b: 2, c: 2982485, output: [7] })
// (BXL(5), State { instruction: 4, a: 5964970, b: 7, c: 2982485, output: [7] })
// (CDV(B), State { instruction: 6, a: 5964970, b: 7, c: 46601, output: [7] })
// (ADV(Three), State { instruction: 8, a: 745621, b: 7, c: 46601, output: [7] })
// (BXC, State { instruction: 10, a: 745621, b: 46606, c: 46601, output: [7] })
// (BXL(6), State { instruction: 12, a: 745621, b: 46600, c: 46601, output: [7] })
// (OUT(B), State { instruction: 14, a: 745621, b: 46600, c: 46601, output: [7, 0] })
// (JNZ(0), State { instruction: 0, a: 745621, b: 46600, c: 46601, output: [7, 0] })
// (BST(A), State { instruction: 2, a: 745621, b: 5, c: 46601, output: [7, 0] })
// (BXL(5), State { instruction: 4, a: 745621, b: 0, c: 46601, output: [7, 0] })
// (CDV(B), State { instruction: 6, a: 745621, b: 0, c: 745621, output: [7, 0] })
// (ADV(Three), State { instruction: 8, a: 93202, b: 0, c: 745621, output: [7, 0] })
// (BXC, State { instruction: 10, a: 93202, b: 745621, c: 745621, output: [7, 0] })
// (BXL(6), State { instruction: 12, a: 93202, b: 745619, c: 745621, output: [7, 0] })
// (OUT(B), State { instruction: 14, a: 93202, b: 745619, c: 745621, output: [7, 0, 3] })
// (JNZ(0), State { instruction: 0, a: 93202, b: 745619, c: 745621, output: [7, 0, 3] })
// (BST(A), State { instruction: 2, a: 93202, b: 2, c: 745621, output: [7, 0, 3] })
// (BXL(5), State { instruction: 4, a: 93202, b: 7, c: 745621, output: [7, 0, 3] })
// (CDV(B), State { instruction: 6, a: 93202, b: 7, c: 728, output: [7, 0, 3] })
// (ADV(Three), State { instruction: 8, a: 11650, b: 7, c: 728, output: [7, 0, 3] })
// (BXC, State { instruction: 10, a: 11650, b: 735, c: 728, output: [7, 0, 3] })
// (BXL(6), State { instruction: 12, a: 11650, b: 729, c: 728, output: [7, 0, 3] })
// (OUT(B), State { instruction: 14, a: 11650, b: 729, c: 728, output: [7, 0, 3, 1] })
// (JNZ(0), State { instruction: 0, a: 11650, b: 729, c: 728, output: [7, 0, 3, 1] })
// (BST(A), State { instruction: 2, a: 11650, b: 2, c: 728, output: [7, 0, 3, 1] })
// (BXL(5), State { instruction: 4, a: 11650, b: 7, c: 728, output: [7, 0, 3, 1] })
// (CDV(B), State { instruction: 6, a: 11650, b: 7, c: 91, output: [7, 0, 3, 1] })
// (ADV(Three), State { instruction: 8, a: 1456, b: 7, c: 91, output: [7, 0, 3, 1] })
// (BXC, State { instruction: 10, a: 1456, b: 92, c: 91, output: [7, 0, 3, 1] })
// (BXL(6), State { instruction: 12, a: 1456, b: 90, c: 91, output: [7, 0, 3, 1] })
// (OUT(B), State { instruction: 14, a: 1456, b: 90, c: 91, output: [7, 0, 3, 1, 2] })
// ----
// (JNZ(0), State { instruction: 0, a: 1456, b: 90, c: 91, output: [7, 0, 3, 1, 2] })
// (BST(A), State { instruction: 2, a: 1456, b: 0, c: 91, output: [7, 0, 3, 1, 2] })
// (BXL(5), State { instruction: 4, a: 1456, b: 5, c: 91, output: [7, 0, 3, 1, 2] })
// (CDV(B), State { instruction: 6, a: 1456, b: 5, c: 45, output: [7, 0, 3, 1, 2] })
// (ADV(Three), State { instruction: 8, a: 182, b: 5, c: 45, output: [7, 0, 3, 1, 2] })
// (BXC, State { instruction: 10, a: 182, b: 40, c: 45, output: [7, 0, 3, 1, 2] })
// (BXL(6), State { instruction: 12, a: 182, b: 46, c: 45, output: [7, 0, 3, 1, 2] })
// (OUT(B), State { instruction: 14, a: 182, b: 46, c: 45, output: [7, 0, 3, 1, 2, 6] })
// (JNZ(0), State { instruction: 0, a: 182, b: 46, c: 45, output: [7, 0, 3, 1, 2, 6] })
// (BST(A), State { instruction: 2, a: 182, b: 6, c: 45, output: [7, 0, 3, 1, 2, 6] }) // b = a % 8
// (BXL(5), State { instruction: 4, a: 182, b: 3, c: 45, output: [7, 0, 3, 1, 2, 6] }) // b = b xor 5
// (CDV(B), State { instruction: 6, a: 182, b: 3, c: 22, output: [7, 0, 3, 1, 2, 6] }) // c = a / 2**b
// (ADV(Three), State { instruction: 8, a: 22, b: 3, c: 22, output: [7, 0, 3, 1, 2, 6] }) // a = a / 8
// (BXC, State { instruction: 10, a: 22, b: 21, c: 22, output: [7, 0, 3, 1, 2, 6] }) // b = b xor c
// (BXL(6), State { instruction: 12, a: 22, b: 19, c: 22, output: [7, 0, 3, 1, 2, 6] }) // b = b xor 6
// (OUT(B), State { instruction: 14, a: 22, b: 19, c: 22, output: [7, 0, 3, 1, 2, 6, 3] }) // output = b % 8
// (JNZ(0), State { instruction: 0, a: 22, b: 19, c: 22, output: [7, 0, 3, 1, 2, 6, 3] })

// --
// (BST(A), State { instruction: 2, a: 22, b: 6, c: 22, output: [7, 0, 3, 1, 2, 6, 3] })
// (BXL(5), State { instruction: 4, a: 22, b: 3, c: 22, output: [7, 0, 3, 1, 2, 6, 3] })
// (CDV(B), State { instruction: 6, a: 22, b: 3, c: 2, output: [7, 0, 3, 1, 2, 6, 3] })
// (ADV(Three), State { instruction: 8, a: 2, b: 3, c: 2, output: [7, 0, 3, 1, 2, 6, 3] })
// (BXC, State { instruction: 10, a: 2, b: 1, c: 2, output: [7, 0, 3, 1, 2, 6, 3] })
// (BXL(6), State { instruction: 12, a: 2, b: 7, c: 2, output: [7, 0, 3, 1, 2, 6, 3] })
// (OUT(B), State { instruction: 14, a: 2, b: 7, c: 2, output: [7, 0, 3, 1, 2, 6, 3, 7] })
// (JNZ(0), State { instruction: 0, a: 2, b: 7, c: 2, output: [7, 0, 3, 1, 2, 6, 3, 7] })
// (BST(A), State { instruction: 2, a: 2, b: 2, c: 2, output: [7, 0, 3, 1, 2, 6, 3, 7] })
// (BXL(5), State { instruction: 4, a: 2, b: 7, c: 2, output: [7, 0, 3, 1, 2, 6, 3, 7] })
// (CDV(B), State { instruction: 6, a: 2, b: 7, c: 0, output: [7, 0, 3, 1, 2, 6, 3, 7] })
// (ADV(Three), State { instruction: 8, a: 0, b: 7, c: 0, output: [7, 0, 3, 1, 2, 6, 3, 7] })
// (BXC, State { instruction: 10, a: 0, b: 7, c: 0, output: [7, 0, 3, 1, 2, 6, 3, 7] })
// (BXL(6), State { instruction: 12, a: 0, b: 1, c: 0, output: [7, 0, 3, 1, 2, 6, 3, 7] })
// (OUT(B), State { instruction: 14, a: 0, b: 1, c: 0, output: [7, 0, 3, 1, 2, 6, 3, 7, 1] })
// (JNZ(0), State { instruction: 16, a: 0, b: 1, c: 0, output: [7, 0, 3, 1, 2, 6, 3, 7, 1] })

fn decode_instruction(opcode: u8, operand: u8) -> Option<Instruction> {
    match opcode {
        0 => {
            if let Some(combo) = Combo::from_operand(operand) {
                Some(Instruction::ADV(combo))
            } else {
                None
            }
        }
        1 => Some(Instruction::BXL(operand)),
        2 => {
            if let Some(combo) = Combo::from_operand(operand) {
                Some(Instruction::BST(combo))
            } else {
                None
            }
        }
        3 => Some(Instruction::JNZ(operand)),
        4 => Some(Instruction::BXC),
        5 => {
            if let Some(combo) = Combo::from_operand(operand) {
                Some(Instruction::OUT(combo))
            } else {
                None
            }
        }
        6 => {
            if let Some(combo) = Combo::from_operand(operand) {
                Some(Instruction::BDV(combo))
            } else {
                None
            }
        }
        7 => {
            if let Some(combo) = Combo::from_operand(operand) {
                Some(Instruction::CDV(combo))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn apply_instruction(instruction: Instruction, mut state: State) -> State {
    state.instruction += 2;
    match instruction {
        Instruction::ADV(combo) => {
            let denominator = u64::pow(2, combo.get_number(&state) as u32);
            let new_state = state.a / denominator;
            state.a = new_state;
        }
        Instruction::BXL(operand) => {
            state.b = state.b ^ operand as u64;
        }
        Instruction::BST(combo) => {
            state.b = combo.get_number(&state) % 8;
        }
        Instruction::JNZ(operand) => {
            if state.a != 0 {
                state.instruction = operand as usize;
            }
        }
        Instruction::BXC => {
            state.b = state.b ^ state.c;
        }
        Instruction::OUT(combo) => {
            let item = combo.get_number(&state) % 8;
            state.output.push(item as u8);
        }
        Instruction::BDV(combo) => {
            let denominator = u64::pow(2, combo.get_number(&state) as u32);
            state.b = state.a / denominator;
        }
        Instruction::CDV(combo) => {
            let denominator = u64::pow(2, combo.get_number(&state) as u32);
            state.c = state.a / denominator;
        }
    }
    state
}

impl Combo {
    fn get_number(&self, state: &State) -> u64 {
        match self {
            Combo::Zero => 0,
            Combo::One => 1,
            Combo::Two => 2,
            Combo::Three => 3,
            Combo::A => state.a,
            Combo::B => state.b,
            Combo::C => state.c,
        }
    }
    fn from_operand(operand: u8) -> Option<Self> {
        match operand {
            0 => Some(Self::Zero),
            1 => Some(Self::One),
            2 => Some(Self::Two),
            3 => Some(Self::Three),
            4 => Some(Self::A),
            5 => Some(Self::B),
            6 => Some(Self::C),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::day17::{apply_instruction, Combo, decode_instruction, Instruction, run_program, State};

    #[test]
    fn adv_test() {
        let state = apply_instruction(Instruction::ADV(Combo::One), State {
            instruction: 0,
            a: 4,
            b: 0,
            c: 0,
            output: vec![],
        });
        assert_eq!(state, State {
            instruction: 2,
            a: 2,
            b: 0,
            c: 0,
            output: vec![],
        })
    }

    #[test]
    fn simple_1() {
        let inst = decode_instruction(2, 6).unwrap();
        let state = apply_instruction(inst, State {
            instruction: 0,
            a: 0,
            b: 0,
            c: 9,
            output: vec![],
        });
        assert_eq!(state, State {
            instruction: 2,
            a: 0,
            b: 1,
            c: 9,
            output: vec![],
        })
    }

    #[test]
    fn simple_2() {
        let state = run_program(&[5, 0, 5, 1, 5, 4], State {
            instruction: 0,
            a: 10,
            b: 0,
            c: 0,
            output: vec![],
        }, 1000);
        assert_eq!(state, State {
            instruction: 6,
            a: 10,
            b: 0,
            c: 0,
            output: vec![0, 1, 2],
        })
    }

    #[test]
    fn simple_3() {
        let state = run_program(&[0, 1, 5, 4, 3, 0], State {
            instruction: 0,
            a: 2024,
            b: 0,
            c: 0,
            output: vec![],
        }, 1000);
        assert_eq!(state, State {
            instruction: 6,
            a: 0,
            b: 0,
            c: 0,
            output: vec![4, 2, 5, 6, 7, 7, 7, 7, 3, 1, 0],
        })
    }

    #[test]
    fn simple_4() {
        let state = run_program(&[1, 7], State {
            instruction: 0,
            a: 0,
            b: 29,
            c: 0,
            output: vec![],
        }, 1000);
        assert_eq!(state, State {
            instruction: 2,
            a: 0,
            b: 26,
            c: 0,
            output: vec![],
        })
    }

    #[test]
    fn simple_5() {
        let state = run_program(&[4, 0], State {
            instruction: 0,
            a: 0,
            b: 2024,
            c: 43690,
            output: vec![],
        }, 1000);
        assert_eq!(state, State {
            instruction: 2,
            a: 0,
            b: 44354,
            c: 43690,
            output: vec![],
        })
    }
}