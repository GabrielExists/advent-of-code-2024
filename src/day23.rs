#![allow(unused_mut, unused_variables, dead_code)]

use std::collections::{HashMap, HashSet};
use itertools::Itertools;
use crate::app::{DayOutput, Diagnostic, Tab};

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
    let mut connection_map = HashMap::<&str, Vec<&str>>::new();
    for (left, right) in pairs.iter() {
        let entry = connection_map.entry(left).or_insert(Vec::new());
        entry.push(right);
        let entry = connection_map.entry(right).or_insert(Vec::new());
        entry.push(left);
    }
    let historian_triplets = silver(&mut tabs, &connection_map);

    DayOutput {
        silver_output: format!("{}", historian_triplets.len()),
        gold_output: format!("{}", 0),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
    }
}

fn silver<'a>(tabs: &mut Vec<Tab>, connection_map: &HashMap<&'a str, Vec<&'a str>>) -> Vec<(&'a str, &'a str, &'a str)> {
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
        strings: triplets.iter().sorted().map(|triplet| format!("{:?}", triplet)).collect(),
        grid: vec![],
    });
    // Triplets starting with t
    let character = 't';
    let historian_triplets = triplets.clone().into_iter().filter(|triplet| {
        let (first, second, third) = &triplet;
        first.starts_with(character) || second.starts_with(character) || third.starts_with(character)
    }).collect::<Vec<_>>();
    tabs.push(Tab {
        title: "Historian triplets".to_string(),
        strings: historian_triplets.iter().sorted().map(|triplet| format!("{:?}", triplet)).collect(),
        grid: vec![],
    });
    historian_triplets
}
