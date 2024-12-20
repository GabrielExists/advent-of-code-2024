use std::borrow::BorrowMut;
use std::fmt::{Display, Formatter};
use log::__private_api::loc;
use slab_tree::{NodeId, NodeMut, NodeRef, Tree, TreeBuilder};
use crate::app::{DayOutput, Diagnostic, Tab};

struct TreePrinter<'a> {
    tree: &'a Tree<char>,
}
#[derive(Clone, Debug)]
struct Output {
    slices: Vec<String>,
}

pub fn puzzle(input: &str) -> DayOutput {
    let mut split = input.split("\n\n");
    let (patterns, designs) = if let (Some(input_patterns), Some(input_designs)) = (split.next(), split.next()) {
        let patterns = input_patterns.split(", ").into_iter().collect::<Vec<_>>();
        let designs = input_designs.split("\n").into_iter().filter(|a| !a.is_empty()).collect::<Vec<_>>();
        (patterns, designs)
    } else {
        (Vec::new(), Vec::new())
    };

    let mut errors: Vec<String> = Vec::new();
    let mut tabs = vec![];
    let mut tree = TreeBuilder::new().with_root('.').build();
    for pattern in patterns.iter() {
        add_string(&mut tree, pattern);
    }

    // let mut options = Vec::new();
    let results = designs.iter().map(|design| {
        // let outputs = traverse(&tree, *design, &mut options, Vec::new());
        let outputs = traverse(&tree, *design);
        (design, outputs)
    }).collect::<Vec<_>>();
    // let num_passing = results.iter().filter(|(_, outputs)| !outputs.is_empty()).count();
    let num_passing = results.iter().filter(|(_, passes)| *passes).count();


    let tree_view = format!("{}", TreePrinter { tree: &tree }).replace(" ", ".");

    tabs.push(Tab {
        title: "Tree view".to_string(),
        strings: tree_view.split("\n").into_iter().map(|item| item.to_string()).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Patterns".to_string(),
        strings: patterns.into_iter().map(|pattern| pattern.to_string()).collect(),
        grid: vec![],
    });
    // tabs.push(Tab {
    //     title: "Options".to_string(),
    //     strings: options.into_iter().map(|item| item.to_string()).collect(),
    //     grid: vec![],
    // });
    tabs.push(Tab {
        title: "Results".to_string(),
        strings: results.into_iter().map(|(design, passes)| format!("{design}: {passes:?}")).collect(),
        grid: vec![],
    });
    DayOutput {
        silver_output: format!("{}", num_passing),
        gold_output: format!("{}", 0),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
    }
}

fn traverse_enumerate(tree: &Tree<char>, haystack: &str, mut accumulated_slices: Vec<String>) -> Vec<Output> {
    match get_slices(tree, haystack) {
        None => {
            vec![]
        }
        Some(slices) => {
            let mut outputs = Vec::new();
            for slice in slices {
                let local_accumulated_slices = accumulated_slices.clone().into_iter().chain([haystack[..slice].to_string()]).collect();
                if slice == haystack.len() {
                    // options.push(format!("Completed"));
                    return vec![Output {
                        slices: accumulated_slices,
                    }]
                }
                // options.push(format!("Slicing {} from {}, {} remains", &haystack[..slice], haystack, &haystack[slice..]));
                let new_outputs = traverse_enumerate(tree, &haystack[slice..], local_accumulated_slices);
                if !new_outputs.is_empty() {
                    outputs.extend(new_outputs.into_iter());
                }
            }
            outputs
        }
    }
}
fn traverse(tree: &Tree<char>, haystack: &str) -> bool {
    match get_slices(tree, haystack) {
        None => {
            false
        }
        Some(slices) => {
            for slice in slices {
                // let local_accumulated_slices = accumulated_slices.clone().into_iter().chain([haystack[..slice].to_string()]).collect();
                if slice == haystack.len() {
                    return true
                }
                // options.push(format!("Slicing {} from {}, {} remains", &haystack[..slice], haystack, &haystack[slice..]));
                let passed = traverse(tree, &haystack[slice..]);
                if passed {
                    return true;
                }
            }
            false
        }
    }
}

fn get_slices(tree: &Tree<char>, haystack: &str) -> Option<Vec<usize>> {
    // options.push(format!("getting slices for {}", haystack));
    let mut node_id = tree.root_id();
    let mut slices = Vec::new();
    let mut index = 0;
    let mut iter = haystack.chars();
    while node_id.is_some() {
        let haystack_char = iter.next();
        let node = tree.get(node_id?)?;
        node_id = None;
        for child in node.children() {
            let node_char = *child.data();
            if node_char == '.' {
                slices.push(index);
            } else if let Some(haystack_char) = haystack_char {
                if haystack_char == node_char {
                    node_id = Some(child.node_id())
                }
            }
        }
        index += 1;
    }
    //rrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrr
    // options.push(format!("slices: {:?}", slices));
    return Some(slices);
}

fn add_string(tree: &mut Tree<char>, string: &str) -> Option<NodeId> {
    let mut node_id = tree.root_id();
    for character in string.chars() {
        node_id = get_or_insert_child(tree, node_id, character);
    }
    node_id = get_or_insert_child(tree, node_id, '.');
    node_id
}

fn get_or_insert_child<'a>(tree: &mut Tree<char>, parent_id: Option<NodeId>, child_character: char) -> Option<NodeId> {
    let parent = tree.get(parent_id?)?;
    match parent.first_child().map(|node_ref| node_ref.node_id()) {
        None => {
            append_child(tree, parent.node_id(), child_character)
        }
        Some(mut child_id) => {
            loop {
                let child = tree.get(child_id)?;
                if *child.data() == child_character {
                    break Some(child.node_id());
                } else {
                    match child.next_sibling() {
                        None => {
                            break append_child(tree, parent.node_id(), child_character);
                        }
                        Some(sibling) => {
                            child_id = sibling.node_id();
                        }
                    }
                }
            }
        }
    }
}

fn append_child(tree: &mut Tree<char>, parent_id: NodeId, child_character: char) -> Option<NodeId> {
    let mut node_mut = tree.get_mut(parent_id)?;
    Some(node_mut.append(child_character).node_id())
}

impl Display for TreePrinter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.tree.write_formatted(f)
    }
}