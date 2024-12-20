use std::collections::HashMap;
use crate::app::{DayOutput, Diagnostic, Tab};
use indextree::{Arena, NodeId};

// struct TreePrinter<'a> {
//     tree: &'a Tree<char>,
// }
// #[derive(Clone, Debug)]
// struct Output {
//     slices: Vec<String>,
// }

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
    let mut arena = Arena::new();
    let root_id = arena.new_node('-');
    for pattern in patterns.iter() {
        add_string(&mut arena, root_id, pattern);
    }

    let mut memo = HashMap::new();
    // let mut options = Vec::new();
    let results = designs.iter().map(|design| {
        // let outputs = traverse(&tree, *design, &mut options, Vec::new());
        let outputs = traverse(&arena, root_id, *design, &mut memo);
        (design, outputs)
    }).collect::<Vec<_>>();
    // let num_passing = results.iter().filter(|(_, outputs)| !outputs.is_empty()).count();
    let num_passing = results.iter().filter(|(_, passes)| *passes).count();


    // let tree_view = format!("{}", TreePrinter { tree: &tree }).replace(" ", ".");

    // tabs.push(Tab {
    //     title: "Tree view".to_string(),
    //     strings: tree_view.split("\n").into_iter().map(|item| item.to_string()).collect(),
    //     grid: vec![],
    // });
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

// fn traverse_enumerate(tree: &Tree<char>, haystack: &str, mut accumulated_slices: Vec<String>) -> Vec<Output> {
//     match get_slices(tree, root, haystack) {
//         None => {
//             vec![]
//         }
//         Some(slices) => {
//             let mut outputs = Vec::new();
//             for slice in slices {
//                 let local_accumulated_slices = accumulated_slices.clone().into_iter().chain([haystack[..slice].to_string()]).collect();
//                 if slice == haystack.len() {
//                     // options.push(format!("Completed"));
//                     return vec![Output {
//                         slices: accumulated_slices,
//                     }]
//                 }
//                 // options.push(format!("Slicing {} from {}, {} remains", &haystack[..slice], haystack, &haystack[slice..]));
//                 let new_outputs = traverse_enumerate(tree, &haystack[slice..], local_accumulated_slices);
//                 if !new_outputs.is_empty() {
//                     outputs.extend(new_outputs.into_iter());
//                 }
//             }
//             outputs
//         }
//     }
// }
fn traverse<'a>(arena: &Arena<char>, root_id: NodeId, haystack: &'a str, memo: &mut HashMap<&'a str, bool>) -> bool {
    if let Some(output) = memo.get(haystack) {
        return *output;
    }
    let output = match get_slices(arena, root_id, haystack) {
        None => {
            false
        }
        Some(slices) => {
            'block: loop {
                for slice in slices {
                    // let local_accumulated_slices = accumulated_slices.clone().into_iter().chain([haystack[..slice].to_string()]).collect();
                    if slice == haystack.len() {
                        break 'block true;
                    }
                    // options.push(format!("Slicing {} from {}, {} remains", &haystack[..slice], haystack, &haystack[slice..]));
                    let passed = traverse(arena, root_id, &haystack[slice..], memo);
                    if passed {
                        break 'block true;
                    }
                }
                break 'block false;
            }
        }
    };
    memo.insert(haystack, output);
    output
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


// impl Display for TreePrinter<'_> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         self.tree.write_formatted(f)
//     }
// }