#![allow(unused_labels, dead_code, unused_mut)]
use crate::app::{DayOutput, Diagnostic, Tab};
use crate::common;
use crate::common::combine_4;
use itertools::Itertools;
use regex::{Match, Regex};
use std::collections::{HashMap, HashSet};

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
    let manually_flipped_pairs: Vec<(&str, &str)> = vec![
        // ("z34", "z33"),
        ("hmk", "z16"),
        ("rvf", "tpc"),
        ("z20", "fhp"),
        ("fcd", "z33"),
        // ("erroneous", "tsc")
        // ("krs", "cpm"),
        // ("gpr", "z10"),
        // ("z21", "nks"),
        // ("ghp", "z33"),


    ];
    for (first, gate, second, output) in logic_gates.iter() {
        let mut output = *output;
        for pair in manually_flipped_pairs.iter() {
            if pair.0 == output {
                output = pair.1;
                break;
            } else if pair.1 == output {
                output = pair.0;
                break;
            }
        }
        // let output =
        // if *output == "krs" {
        //     "cpm"
        // } else if *output == "cpm" {
        //     "krs"
        // } else if *output == "gpr" {
        //     "z10"
        // } else if *output == "z10" {
        //     "gpr"
        // } else if *output == "z21" {
        //     "nks"
        // } else if *output == "nks" {
        //     "z21"
        // } else if *output == "ghp" {
        //     "z33"
        // } else if *output == "z33" {
        //     "ghp"
        // } else {
        //     *output
        // }
        ;
        mapping.insert(output, Terminal::Gate(first, *gate, second));
    }

    let re_input_x = Regex::new(r"x(?P<num>\d*)").expect("Should compile");
    let re_input_y = Regex::new(r"y(?P<num>\d*)").expect("Should compile");
    let re_output = Regex::new(r"z(?P<num>\d*)").expect("Should compile");
    let mut memoize = HashMap::<&str, bool>::new();
    let silver = calculate_output(&mut errors, &mapping, &mut memoize);

    // let plantuml_lines = create_plantuml(
    //     &mut errors,
    //     &mapping,
    //     &re_input_x,
    //     &re_input_y,
    //     &re_output,
    //     &mut memoize,
    // );

    // {
    //     let mapping = mapping.clone();
    // }
    // let influence_map = influence_map_from_mapping(&mapping);
    // tabs.push(Tab {
    //     title: "Influence map".to_string(),
    //     strings:
    //         .iter()
    //         .sorted()
    //         .map(|tuple| format!("{:?}", tuple))
    //         .collect(),
    //     grid: vec![],
    // });

    let mut influences_tab = Vec::new();
    let mut input_influences_tab = Vec::new();
    for i in 0..46 {
        let terminal = format!("z{:02}", i);
        match get_influences(&mut errors, &mapping, &terminal, Vec::new()) {
            Ok(influences) => {
                influences_tab.push(format!(
                    "{}: {:?}",
                    terminal,
                    influences.iter().sorted().join(", ")
                ));
                input_influences_tab.push(format!(
                    "{}: {:?}",
                    terminal,
                    influences
                        .iter()
                        .filter(|name| { re_input_x.is_match(name) || re_input_y.is_match(name) })
                        .sorted()
                        .join(", ")
                ));
            }
            Err(error) => {
                errors.push(error);
            }
        }
    }
    tabs.push(Tab {
        title: "Influences".to_string(),
        strings: influences_tab,
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Input influences".to_string(),
        strings: input_influences_tab,
        grid: vec![],
    });

    let mut tests = Vec::new();
    for i in 0..15 {
        for j in 0..15 {
            let expectation = i & j;
            let output = calculate_output_given_inputs(&mut errors, &mapping, i, j);
            let correct_bits = get_correct_bits(expectation, output, 45);
            tests.push(format!(
                "{} & {} = {}, {} correct, {:b}, {:b}",
                i, j, output, correct_bits, output, expectation
            ));
        }
    }

    let mut incremental_comparison = Vec::new();
    let mut last_ok_bit = 1;
    let mut last_failing_bit = None;
    let mut disruptions: Vec<(u64, u64, Vec<&str>)> = Vec::new();
    let mut disruption_ranges: Vec<(u64, u64)> = Vec::new();
    let mut disruption_candidates: Vec<&str> = Vec::new();
    let mut reference_scores: HashMap<u64, i64> = HashMap::new();
    for bits in 2..45_u64 {
        let mut bit_ok = true;
        let score = reference_scores.entry(bits).or_insert(0);
        'x_loop: for x in 0..4 {
            for y in 0..4 {
                // Shift this to affect the current bit and the one below
                let x = x << (bits - 2);
                let y = y << (bits - 2);
                // for x in 0..16 {
                //     for y in 0..16 {
                let expected = x + y;
                let output = calculate_output_given_inputs(&mut errors, &mapping, x, y);
                let comparison_bits = 45;
                let correct_bits = get_correct_bits(expected, output, comparison_bits);
                *score += correct_bits as i64;
                if correct_bits != comparison_bits {
                    if bit_ok {
                        incremental_comparison.push(format!(
                            "Bit {} not ok. x {x:b}, y {y:b}, expected {expected:b}, output {output:b}",
                            bits
                        ));
                    }
                    bit_ok = false;
                }
                // incremental_comparison.push(format!("{} bits, {} + {} = {}, expected {}, {}/{} bits ok", bits, x, y, output, expected, correct_bits, bits));
            }
        }
        if bit_ok {
            if let Some(failing_bit) = last_failing_bit {
                let diff = get_influence_diff(&mut errors, &mapping, last_ok_bit, failing_bit);
                incremental_comparison.push(format!("{:?}", diff));
                disruptions.push((last_ok_bit, failing_bit, diff.clone()));
                disruption_ranges.push((last_ok_bit, failing_bit));
                disruption_candidates.extend(diff);
                last_failing_bit = None;
            }
            last_ok_bit = bits;
        } else {
            last_failing_bit = Some(bits);
        }
    }

    let mut ok_swaps: Vec<(Vec<(&str, &str)>, u64, u64)> = Vec::new();
    let mut nok_swaps: Vec<(Vec<(&str, &str)>, u64, u64)> = Vec::new();

    let mut score_deltas: HashMap<&str, HashMap<u64, i64>> = HashMap::new();
    // 'swap_pairs: for swap_pairs in find_swap_pairs_counters(&disruptions) {
    // 'swap_pairs: for swap_pairs in find_swap_pairs_manual() {
    //     errors.push(format!("{:?}", swap_pairs));
    // e
    if false {
        // 'swap_pairs: for swap_pairs in find_swap_pairs_manual() {
        'swap_pairs: for swap_pairs in find_swap_single(&mapping) {
            // for swap_pairs in find_swap_pairs(&disruption_candidates) {
            let mut mapping = mapping.clone();
            for (first, second) in swap_pairs.iter() {
                // }
                // for (first_index, first) in disruption_candidates.iter().enumerate() {
                //     for (second_index, second) in disruption_candidates.iter().enumerate() {
                // if first_index == second_index {
                //     continue;
                // }
                if let Some(first_terminal) = mapping.get(first).cloned() {
                    if let Some(second_terminal) = mapping.get(second).cloned() {
                        mapping
                            .entry(second)
                            .and_modify(|entry| *entry = first_terminal);
                        mapping
                            .entry(first)
                            .and_modify(|entry| *entry = second_terminal);
                        if let Err(error) = get_influences(&mut errors, &mapping, first, Vec::new())
                        {
                            errors.push(format!("Skipped {} to {}, {}", first, second, error));
                            continue 'swap_pairs;
                        }
                        if let Err(error) =
                            get_influences(&mut errors, &mapping, second, Vec::new())
                        {
                            errors
                                .push(format!("Skipped {} to {} second, {}", first, second, error));
                            continue 'swap_pairs;
                        }
                    } else {
                        errors.push(format!("No terminal called {} found", second));
                    }
                } else {
                    errors.push(format!("No terminal called {} found", first));
                }
            }

            for (last_ok, last_failing) in disruption_ranges.iter() {
                let mut check_ok = true;
                // 'complete_check: for bits in 0..45 {
                'complete_check: for bits in *last_ok..*last_failing {
                    // let mut score = 0;
                    for x in 0..4 {
                        for y in 0..4 {
                            // Shift this to affect the current bit and the one below
                            let x = x << (bits - 2);
                            let y = y << (bits - 2);
                            // for x in 0..16 {
                            //     for y in 0..16 {
                            let expected = x + y;
                            let output = calculate_output_given_inputs(&mut errors, &mapping, x, y);
                            let comparison_bits = 45;
                            let correct_bits = get_correct_bits(expected, output, comparison_bits);
                            // score += correct_bits;
                            if correct_bits != comparison_bits {
                                // incremental_comparison.push(format!(
                                //     "Bit {} not ok. x {x:b}, y {y:b}, expected {expected:b}, output {output:b}",
                                //     bits
                                // ));
                                check_ok = false;
                                break 'complete_check;
                            }
                            // incremental_comparison.push(format!("{} bits, {} + {} = {}, expected {}, {}/{} bits ok", bits, x, y, output, expected, correct_bits, bits));
                        }
                    }
                    // let delta = score_deltas
                    //     .entry(first)
                    //     .or_insert(HashMap::new())
                    //     .entry(bits)
                    //     .or_insert(0i64);
                    // let reference_score = reference_scores[&bits];
                    // *delta += reference_score - score as i64;
                }
                if check_ok {
                    ok_swaps.push((swap_pairs.clone(), *last_ok, *last_failing));
                    // break 'swap_pairs;
                } else {
                    nok_swaps.push((swap_pairs.clone(), *last_ok, *last_failing));
                }
            }
        }
    }
    let gold = manually_flipped_pairs
        .iter()
        .flat_map(|pair| [pair.0, pair.1])
        // let gold = [
        //     "cpm",
        //     "krs",
        //     "z10",
        //     "gpr",
        //     "nks",
        //     "z21",
        //     "z33",
        //     "ghp",
        // ]
        //     .iter()
        //     .copied()
        .chain(
            ok_swaps
                .first()
                .cloned()
                .map(|a: (Vec<(&str, &str)>, u64, u64)| a.0)
                .unwrap_or(Vec::new())
                .into_iter()
                .flat_map(|a| [a.0, a.1].into_iter()),
        )
        .sorted()
        .join(",");
    // for (last_ok, last_failing, terminals) in disruptions {
    //     for (first_index, first) in terminals.iter().enumerate() {
    //         for (second_index, second) in terminals.iter().enumerate() {
    //             if first_index == second_index {
    //                 continue;
    //             }
    //             let mut mapping = mapping.clone();
    //             let first_terminal = mapping[first];
    //             let second_terminal = mapping[second];
    //             mapping
    //                 .entry(second)
    //                 .and_modify(|entry| *entry = first_terminal);
    //             mapping
    //                 .entry(first)
    //                 .and_modify(|entry| *entry = second_terminal);
    //             if let Err(error) = get_influences(&mut errors, &mapping, first, Vec::new()) {
    //                 errors.push(format!("Skipped {} to {}, {}", first, second, error));
    //                 continue;
    //             }
    //             if let Err(error) = get_influences(&mut errors, &mapping, second, Vec::new()) {
    //                 errors.push(format!("Skipped {} to {} second, {}", first, second, error));
    //                 continue;
    //             }
    //
    //             let mut check_ok = true;
    //             'complete_check: for bits in last_ok..=last_failing {
    //                 for x in 0..4 {
    //                     for y in 0..4 {
    //                         // Shift this to affect the current bit and the one below
    //                         let x = x << (bits - 2);
    //                         let y = y << (bits - 2);
    //                         // for x in 0..16 {
    //                         //     for y in 0..16 {
    //                         let expected = x + y;
    //                         let output = calculate_output_given_inputs(&mut errors, &mapping, x, y);
    //                         let comparison_bits = 45;
    //                         let correct_bits = get_correct_bits(expected, output, comparison_bits);
    //                         if correct_bits != comparison_bits {
    //                             // incremental_comparison.push(format!(
    //                             //     "Bit {} not ok. x {x:b}, y {y:b}, expected {expected:b}, output {output:b}",
    //                             //     bits
    //                             // ));
    //                             check_ok = false;
    //                             break 'complete_check;
    //                         }
    //                         // incremental_comparison.push(format!("{} bits, {} + {} = {}, expected {}, {}/{} bits ok", bits, x, y, output, expected, correct_bits, bits));
    //                     }
    //                 }
    //             }
    //             if check_ok {
    //                 ok_swaps.push((first, second));
    //             }
    //         }
    //     }
    // }

    tabs.push(Tab {
        title: "Tests".to_string(),
        strings: tests,
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Incremental comparison".to_string(),
        strings: incremental_comparison,
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Ok swaps".to_string(),
        strings: ok_swaps.iter().map(|a| format!("{:?}", a)).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Nok swaps".to_string(),
        strings: nok_swaps.iter().map(|a| format!("{:?}", a)).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Reference scores".to_string(),
        strings: reference_scores
            .iter()
            .sorted_by_key(|(bits, _)| **bits)
            .map(|(bits, score)| format!("{} {}", *bits, *score))
            .collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Score deltas".to_string(),
        strings: score_deltas
            .iter()
            .map(|(terminal_name, submap)| {
                let list = submap
                    .iter()
                    .sorted_by_key(|(a, _)| **a)
                    .map(|(_, b)| *b)
                    .join(",");
                format!("{}: {}", terminal_name, list)
            })
            .collect(),
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
    // tabs.push(Tab {
    //     title: "Plantuml".to_string(),
    //     strings: plantuml_lines,
    //     grid: vec![],
    // });
    tabs.push(Tab {
        title: "Errors".to_string(),
        strings: errors,
        grid: vec![],
    });

    DayOutput {
        silver_output: format!("{}", silver),
        gold_output: gold,
        diagnostic: Diagnostic::with_tabs(tabs, "".to_string()),
    }
}

fn find_swap_pairs_manual() -> Vec<Vec<(&'static str, &'static str)>> {
    // let disruption_candidates = vec!["trf", "tkq", "x34", "bwd", "hsv"];
    // let disruption_candidates = vec!["ghp", "trf", "nks", "jtg", "tkq", "bwd", "x34"];
    let disruption_candidates = vec!["hmk", "fcd", "z16"];
    let mut swap_pairs = Vec::new();
    for (a_first_index, a_first) in disruption_candidates.iter().enumerate() {
        for a_second in disruption_candidates.iter().skip(a_first_index + 1) {
            for (b_first_index, b_first) in disruption_candidates.iter().enumerate() {
                if b_first != a_first && b_first != a_second {
                    for b_second in disruption_candidates.iter().skip(b_first_index + 1) {
                        if b_second != a_first && b_second != a_second {
                            // for (c_first_index, c_first) in disruption_candidates.iter().enumerate()
                            // {
                            //     if c_first != a_first
                            //         && c_first != a_second
                            //         && c_first != b_first
                            //         && c_first != b_second
                            //     {
                            //         for c_second in
                            //             disruption_candidates.iter().skip(c_first_index + 1)
                            //         {
                            //             if c_second != a_first
                            //                 && c_second != a_second
                            //                 && c_second != b_first
                            //                 && c_second != b_second
                            //             {
                            swap_pairs.push(vec![
                                (*a_first, *a_second),
                                (*b_first, *b_second),
                                // (*c_first, *c_second),
                            ]);
                            //             }
                            //         }
                            //     }
                            // }
                        }
                    }
                }
            }
        }
    }
    swap_pairs
}
fn find_swap_pairs_counters<'a>(
    disruptions: &Vec<(u64, u64, Vec<&'a str>)>,
) -> Vec<Vec<(&'a str, &'a str)>> {
    let mut swap_pairs = Vec::new();
    if let Some(collection_one) = &disruptions.get(0) {
        let collection_one = &collection_one.2;
        if let Some(collection_two) = &disruptions.get(1) {
            let collection_two = &collection_two.2;
            if let Some(collection_three) = &disruptions.get(2) {
                let collection_three = &collection_three.2;
                for a_first in collection_one.iter() {
                    for a_second in collection_two.iter().chain(collection_three.iter()) {
                        for b_first in collection_two.iter() {
                            if b_first != a_first && b_first != a_second {
                                // for b_second in disruption_candidates[b_first_index + 1..].iter() {
                                for b_second in collection_three.iter().chain(collection_one.iter())
                                {
                                    if b_second != a_first && b_second != a_second {
                                        for c_first in collection_three.iter() {
                                            if c_first != a_first
                                                && c_first != a_second
                                                && c_first != b_first
                                                && c_first != b_second
                                            {
                                                // for c_second in disruption_candidates[c_first_index + 1..].iter() {
                                                for c_second in collection_one
                                                    .iter()
                                                    .chain(collection_two.iter())
                                                {
                                                    if c_second != a_first
                                                        && c_second != a_second
                                                        && c_second != b_first
                                                        && c_second != b_second
                                                    {
                                                        swap_pairs.push(vec![
                                                            (*a_first, *a_second),
                                                            (*b_first, *b_second),
                                                            (*c_first, *c_second),
                                                        ]);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    swap_pairs
}

fn find_swap_pairs<'a>(disruption_candidates: &Vec<&'a str>) -> Vec<Vec<(&'a str, &'a str)>> {
    let mut swap_pairs = Vec::new();
    for (a_first_index, a_first) in disruption_candidates.iter().enumerate() {
        for a_second in disruption_candidates[a_first_index + 1..].iter() {
            for (b_first_index, b_first) in disruption_candidates.iter().enumerate() {
                if b_first != a_first && b_first != a_second {
                    for b_second in disruption_candidates[b_first_index + 1..].iter() {
                        if b_second != a_first && b_second != a_second {
                            for (c_first_index, c_first) in disruption_candidates.iter().enumerate()
                            {
                                if c_first != a_first
                                    && c_first != a_second
                                    && c_first != b_first
                                    && c_first != b_second
                                {
                                    for c_second in
                                        disruption_candidates[c_first_index + 1..].iter()
                                    {
                                        if c_second != a_first
                                            && c_second != a_second
                                            && c_second != b_first
                                            && c_second != b_second
                                        {
                                            swap_pairs.push(vec![
                                                (*a_first, *a_second),
                                                (*b_first, *b_second),
                                                (*c_first, *c_second),
                                            ]);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    swap_pairs
}

fn find_swap_single<'a>(mapping: &HashMap<&'a str, Terminal>) -> Vec<Vec<(&'a str, &'a str)>> {
    let mut swap_pairs = Vec::new();
    for (a_first_index, a_first) in mapping.keys().enumerate() {
        for a_second in mapping.keys().skip(a_first_index + 1) {
            swap_pairs.push(vec![(*a_first, *a_second)]);
        }
    }
    swap_pairs
}

fn get_influence_diff<'a>(
    errors: &mut Vec<String>,
    mapping: &HashMap<&'a str, Terminal<'a>>,
    last_ok_bit: u64,
    last_failing_bit: u64,
) -> Vec<&'a str> {
    let ok_name = format!("z{:02}", last_ok_bit);
    let failing_name = format!("z{:02}", last_failing_bit);
    if let Ok(ok_names) = get_influences(errors, mapping, &ok_name, Vec::new()) {
        if let Ok(failing_names) = get_influences(errors, mapping, &failing_name, Vec::new()) {
            let mut result = Vec::new();
            for failing_name in failing_names {
                if !ok_names.contains(&failing_name) {
                    result.push(failing_name);
                }
            }
            return result;
        }
    }
    Vec::new()
}

fn get_correct_bits(expectation: u64, actual: u64, num_bits: u64) -> u64 {
    let mut num_correct_bits = 0;
    for i in 0..num_bits {
        if expectation & (1 << i) == actual & (1 << i) {
            num_correct_bits += 1;
        }
    }
    num_correct_bits
}
// fn influence_map_from_mapping<'a>(mapping: &HashMap<&'a str, Terminal<'a>>) -> HashMap<&'a str, Vec<&'a str>> {
//     let mut influence_map = HashMap::new();
//     for (terminal_name, terminal) in mapping.iter() {
//         match terminal {
//             Terminal::Bool(_) => {}
//             Terminal::Gate(left, _, right) => {
//                 influence_map.entry(*terminal_name).or_insert(Vec::new()).push(*left);
//                 influence_map.entry(*terminal_name).or_insert(Vec::new()).push(*right);
//             }
//         }
//     }
//     influence_map
// }
fn get_influences<'a, 'b: 'a>(
    errors: &mut Vec<String>,
    mapping: &HashMap<&'b str, Terminal<'b>>,
    subject: &'a str,
    mut visited: Vec<&'a str>,
) -> Result<HashSet<&'b str>, String> {
    if visited.contains(&subject) {
        return Err(format!("Cycled, on {}", subject));
    }
    let mut influences = HashSet::new();
    visited.push(subject);
    for influence in mapping
        .get(subject)
        .cloned()
        .map(|terminal| {
            if let Terminal::Gate(left, _gate, right) = terminal {
                // errors.push(format!("Found gate {} {}", left, right));
                vec![left, right]
            } else {
                // errors.push(format!("Found non-gate"));
                Vec::new()
            }
        })
        .unwrap_or_else(|| {
            errors.push(format!("Couldn't find terminal {}", subject));
            Vec::new()
        })
    {
        influences.insert(influence);
        for new_influence in get_influences(errors, mapping, influence, visited.clone())? {
            influences.insert(new_influence);
        }
    }
    // errors.push(format!("Returning {:?}", influences));
    Ok(influences)
}
// fn get_influences_iterative<'a>(
//     influence_map: &HashMap<&'a str, Terminal<'a>>,
//     subject: &str,
// ) -> Result<Vec<&'a str>, String> {
//     let mut influences = influence_map
//         .get(subject)
//         .cloned()
//         .map(|terminal| {
//             if let Terminal::Gate(left, _gate, right) = terminal {
//                 vec![left, right]
//             } else {
//                 Vec::new()
//             }
//         })
//         .unwrap_or(Vec::new());
//     let mut unhandled_influences = influences.clone();
//     loop {
//         if let Some(current) = unhandled_influences.pop() {
//             let mut new_influences = influence_map
//                 .get(current)
//                 .cloned()
//                 .map(|terminal| {
//                     if let Terminal::Gate(left, _gate, right) = terminal {
//                         vec![left, right]
//                     } else {
//                         Vec::new()
//                     }
//                 })
//                 .unwrap_or(Vec::new());
//             for new_influence in new_influences {
//                 if !influences.contains(&new_influence) {
//                     influences.push(new_influence);
//                     unhandled_influences.push(new_influence);
//                 }
//             }
//         } else {
//             break;
//         }
//     }
//     Ok(influences)
// }

fn calculate_output_given_inputs(
    errors: &mut Vec<String>,
    mapping: &HashMap<&str, Terminal>,
    x: u64,
    y: u64,
) -> u64 {
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
    re_input_x: &Regex,
    re_input_y: &Regex,
    re_output: &Regex,
    memoize: &mut HashMap<&str, bool>,
) -> Vec<String> {
    let mut plantuml_lines = Vec::new();
    plantuml_lines.push("@startuml".to_string());
    plantuml_lines.push("left to right direction".to_string());
    plantuml_lines.push("title Advent of Code 2024 day 24 diagram".to_string());
    for (terminal_name, value) in memoize.iter().sorted() {
        // let group_name = if re_output.is_match(terminal_name) {
        //     "z"
        // } else if re_input_x.is_match(terminal_name) {
        //     "x"
        // } else if re_input_y.is_match(terminal_name) {
        //     "y"
        // } else {
        //     "m"
        // };
        if let Some(terminal) = mapping.get(terminal_name) {
            let kind = match terminal {
                Terminal::Bool(_) => "Input",
                Terminal::Gate(_, gate, _) => match gate {
                    GateType::And => "AND",
                    GateType::Xor => "XOR",
                    GateType::Or => "OR",
                },
            };
            // plantuml_lines.push(format!("map {}.{} {{", group_name, terminal_name));
            plantuml_lines.push(format!("map {} {{", terminal_name));
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
