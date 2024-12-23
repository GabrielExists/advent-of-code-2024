use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crate::app::{DayOutput, Diagnostic, Tab};
use indextree::{Arena, NodeEdge, NodeId};

pub fn puzzle(_input: &str) -> DayOutput {
    let mut errors: Vec<String> = Vec::new();
    let mut tabs: Vec<Tab> = Vec::new();
    DayOutput {
        silver_output: format!("{}", 0),
        gold_output: format!("{}", 0),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
    }
}
