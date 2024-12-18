use regex::Regex;
use crate::app::{DayOutput, Diagnostic};
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
    let mut output_silver: Option<Vec<u8>> = None;
    let mut output_gold = None;
    if let Some(captures) = re.captures(input) {
        let a = capture_parse(&captures, "a");
        let b = capture_parse(&captures, "b");
        let c = capture_parse(&captures, "c");
        let program: Option<Vec<u8>> = captures.name("program").map(|m|{
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

            for i in 0..30000 {
                let mut current_state = input_state.clone();
                current_state.a = i as u64;
                let output_state = run_program(&program, current_state, 1000);
                log::info!("{:?}, {:?}, {:?}", program, i, output_state);
                if output_state.output == program {
                    output_gold = Some(i);
                    break;
                }
            }
        }
    }
    let tabs = Vec::new();
    let errors: Vec<String> = Vec::new();

    let formatted_output = output_silver.map(|list| list.into_iter().map(|number| number.to_string()).collect::<Vec<_>>().join(","));
    DayOutput {
        silver_output: format!("{}", formatted_output.unwrap_or(String::new())),
        gold_output: format!("{}", output_gold.unwrap_or(0)),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
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
        },1000);
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
        },1000);
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
        },1000);
        assert_eq!(state, State {
            instruction: 2,
            a: 0,
            b: 44354,
            c: 43690,
            output: vec![],
        })
    }
}