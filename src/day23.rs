#![allow(unused_mut, unused_variables, dead_code)]

use crate::app::{DayOutput, Diagnostic, Tab};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

pub fn puzzle(input: &str) -> DayOutput {
    let errors: Vec<String> = Vec::new();
    let mut tabs: Vec<Tab> = Vec::new();

    // Parse input
    let pairs = input
        .lines()
        .filter_map(|line| {
            let mut split = line.split("-");
            let left = split.next();
            let right = split.next();
            match (left, right) {
                (Some(left), Some(right)) => Some((left.trim(), right.trim())),
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    tabs.push(Tab {
        title: "Input parsed".to_string(),
        strings: pairs.iter().map(|a| format!("{:?}", a)).collect(),
        grid: vec![],
    });
    // Create a map of connections
    let mut string_connection_map = HashMap::<&str, Vec<&str>>::new();
    let mut connection_map = HashMap::<u32, Vec<u32>>::new();
    let mut next_id = 0;
    let mut lookup = HashMap::<&str, u32>::new();
    for (left, right) in pairs.iter() {
        let left_id = *lookup.entry(left).or_insert({
            let id = next_id;
            next_id += 1;
            id
        });
        let right_id = *lookup.entry(right).or_insert({
            let id = next_id;
            next_id += 1;
            id
        });
        let entry = string_connection_map.entry(left).or_insert(Vec::new());
        entry.push(right);
        let entry = string_connection_map.entry(right).or_insert(Vec::new());
        entry.push(left);

        let entry = connection_map.entry(left_id).or_insert(Vec::new());
        entry.push(right_id);
        entry.sort();
        let entry = connection_map.entry(right_id).or_insert(Vec::new());
        entry.push(left_id);
        entry.sort();
    }
    let historian_triplets = silver(&mut tabs, &string_connection_map);

    let gold_result = gold(&connection_map);
    tabs.push(Tab {
        title: "Gold result as numbers".to_string(),
        strings: gold_result.iter().map(|item| item.to_string()).collect(),
        grid: vec![],
    });

    let reverse_lookup =
        HashMap::<u32, &str>::from_iter(lookup.into_iter().map(|(key, id)| (id, key)));

    let password_parts = gold_result.iter().map(|id| {
        reverse_lookup.get(id).copied().unwrap_or("unknown")
    }).collect::<Vec<_>>();
    tabs.push(Tab {
        title: "Gold result".to_string(),
        strings: password_parts.iter().map(|item|item.to_string()).collect(),
        grid: vec![],
    });
    let password = password_parts.into_iter().sorted().join(",");

    DayOutput {
        silver_output: format!("{}", historian_triplets.len()),
        gold_output: password,
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
    }
}

fn gold(connection_map: &HashMap<u32, Vec<u32>>) -> Vec<u32> {
    let mut memoize = HashMap::new();

    let mut suggestions: Vec<Vec<u32>> = Vec::new();

    let mut new_past_items = Vec::new();
    for host in connection_map.keys().sorted() {
        let result = gold_aux(&mut memoize, connection_map, *host, &new_past_items);
        suggestions.push(result);
    }
    let result = suggestions
        .iter()
        .max_by_key(|item| item.len())
        .map(|item| item.clone())
        .unwrap_or(Vec::new());
    result
}

fn gold_aux(
    memoize: &mut HashMap<(u32, Vec<u32>), Vec<u32>>,
    connection_map: &HashMap<u32, Vec<u32>>,
    current: u32,
    past_items: &Vec<u32>,
) -> Vec<u32> {
    if let Some(list) = memoize.get(&(current, past_items.clone())) {
        return list.clone();
    }
    let connections = &connection_map[&current];
    for past_item in past_items {
        if !connections.contains(past_item) {
            memoize.insert((current, past_items.clone()), Vec::new());
            return Vec::new();
        }
    }
    let mut suggestions: Vec<Vec<u32>> = Vec::new();
    let mut new_past_items = past_items.clone();
    new_past_items.push(current);
    new_past_items.sort();
    for connection in connections {
        if !past_items.contains(connection) {
            let result = gold_aux(memoize, connection_map, *connection, &new_past_items);
            if result.len() > 0 {
                suggestions.push(result);
            }
        }
    }
    let result = if suggestions.is_empty() {
        let mut item = past_items.clone();
        item.push(current);
        item
    } else {
        suggestions
            .iter()
            .max_by_key(|item| item.len())
            .map(|item| {
                let mut item = item.clone();
                item
            })
            .unwrap_or(Vec::new())
    };
    memoize.insert((current, past_items.clone()), result.clone());
    result
}

fn silver<'a>(
    tabs: &mut Vec<Tab>,
    connection_map: &HashMap<&'a str, Vec<&'a str>>,
) -> Vec<(&'a str, &'a str, &'a str)> {
    // Check for groups of three
    let mut triplets = HashSet::<(&str, &str, &str)>::new();
    // fo = first order, so = second order, to = third order
    // Order refers to which
    for (host, fo_connections) in connection_map.iter() {
        let host = *host;
        for (fo_index, second) in fo_connections.into_iter().enumerate() {
            let second = *second;
            let so_connections = &connection_map[second];
            for third in so_connections {
                let third = *third;
                // We already know the host and second are connected
                if third != host {
                    let to_connections = &connection_map[third];
                    for to_connection in to_connections {
                        // If the triangle is closed
                        if *to_connection == host {
                            // Sort and add
                            let triplet = if host < second {
                                if second < third {
                                    (host, second, third)
                                } else if host < third {
                                    (host, third, second)
                                } else {
                                    (third, host, second)
                                }
                            } else {
                                if host < third {
                                    (second, host, third)
                                } else if second < third {
                                    (second, third, host)
                                } else {
                                    (third, second, host)
                                }
                            };
                            triplets.insert(triplet);
                        }
                    }
                }
            }
        }
    }
    tabs.push(Tab {
        title: "All triplets".to_string(),
        strings: triplets
            .iter()
            .sorted()
            .map(|triplet| format!("{:?}", triplet))
            .collect(),
        grid: vec![],
    });

    // Triplets starting with t
    let character = 't';
    let historian_triplets = triplets
        .clone()
        .into_iter()
        .filter(|triplet| {
            let (first, second, third) = &triplet;
            first.starts_with(character)
                || second.starts_with(character)
                || third.starts_with(character)
        })
        .collect::<Vec<_>>();
    tabs.push(Tab {
        title: "Historian triplets".to_string(),
        strings: historian_triplets
            .iter()
            .sorted()
            .map(|triplet| format!("{:?}", triplet))
            .collect(),
        grid: vec![],
    });
    historian_triplets
}
