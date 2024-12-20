use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crate::app::{DayOutput, Diagnostic, Tab};
use indextree::{Arena, NodeEdge, NodeId};

struct ArenaPrinter<'a> {
    arena: &'a Arena<char>,
    root: NodeId,
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

    let errors: Vec<String> = Vec::new();
    let mut tabs = vec![];
    let mut arena = Arena::new();
    let root_id = arena.new_node('-');
    for pattern in patterns.iter() {
        add_string(&mut arena, root_id, pattern);
    }

    let mut memo = HashMap::new();
    let results = designs.iter().map(|design| {
        let possibilities = traverse(&arena, root_id, *design, &mut memo);
        (design, possibilities)
    }).collect::<Vec<_>>();
    let num_passing = results.iter().filter(|(_, possibilities)| *possibilities > 0).count();
    let combined_possibilities: u64 = results.iter().map(|(_, possibilities)| possibilities).sum();


    let tree_view = format!("{}", ArenaPrinter {
        arena: &arena,
        root: root_id,
    }).replace(" ", ".");

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
    tabs.push(Tab {
        title: "Results".to_string(),
        strings: results.into_iter().map(|(design, passes)| format!("{design}: {passes:?}")).collect(),
        grid: vec![],
    });
    DayOutput {
        silver_output: format!("{}", num_passing),
        gold_output: format!("{}", combined_possibilities),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
    }
}

fn traverse<'a>(arena: &Arena<char>, root_id: NodeId, haystack: &'a str, memo: &mut HashMap<&'a str, u64>) -> u64 {
    if let Some(output) = memo.get(haystack) {
        return *output;
    }
    let possibilities = match get_slices(arena, root_id, haystack) {
        None => {
            0
        }
        Some(slices) => {
            'block: loop {
                let mut accumulator = 0;
                for slice in slices {
                    // let local_accumulated_slices = accumulated_slices.clone().into_iter().chain([haystack[..slice].to_string()]).collect();
                    if slice == haystack.len() {
                        accumulator += 1;
                    }
                    // options.push(format!("Slicing {} from {}, {} remains", &haystack[..slice], haystack, &haystack[slice..]));
                    let possibilities = traverse(arena, root_id, &haystack[slice..], memo);
                    accumulator += possibilities;
                }
                break 'block accumulator;
            }
        }
    };
    memo.insert(haystack, possibilities);
    possibilities
}

fn get_slices(arena: &Arena<char>, root: NodeId, haystack: &str) -> Option<Vec<usize>> {
    let mut node_id = Some(root);
    let mut slices = Vec::new();
    let mut index = 0;
    let mut iter = haystack.chars();
    while node_id.is_some() {
        let haystack_char = iter.next();
        let children = node_id?.children(arena);
        node_id = None;
        for child_id in children {
            let node_char = arena.get(child_id)?.get();
            if *node_char == '.' {
                slices.push(index);
            } else if let Some(haystack_char) = haystack_char {
                if haystack_char == *node_char {
                    node_id = Some(child_id);
                }
            }
        }
        index += 1;
    }
    return Some(slices);
}

fn add_string(arena: &mut Arena<char>, root: NodeId, string: &str) -> NodeId {
    let mut node_id = root;
    for current_char in string.chars() {
        node_id = get_or_insert_child(arena, node_id, current_char);
    }
    node_id = get_or_insert_child(arena, node_id, '.');
    node_id
}

fn get_or_insert_child<'a>(arena: &mut Arena<char>, parent_id: NodeId, new_char: char) -> NodeId {
    for child_id in parent_id.children(arena) {
        if let Some(child) = arena.get(child_id) {
            if *child.get() == new_char {
                return child_id;
            }
        }
    }
    let new_node = arena.new_node(new_char);
    parent_id.append(new_node, arena);
    new_node
}


impl Display for ArenaPrinter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut depth: i32 = 0;
        for edge in self.root.traverse(self.arena) {
            match edge {
                NodeEdge::Start(start) => {
                    if let Some(node) = self.arena.get(start) {
                        let character = *node.get();
                        if character == '.' {
                            f.write_str("*")?;
                        } else {
                            let padding = ".".repeat(std::cmp::max(depth - 1, 0) as usize);
                            f.write_fmt(format_args!("\n{}{}", padding, character))?;
                        }
                    }
                    depth += 1;
                }
                NodeEdge::End(_end) => {
                    depth -= 1;
                }
            }
        }
        Ok(())
    }
}