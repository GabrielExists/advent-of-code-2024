use crate::app::{DayOutput, Diagnostic, Tab};

pub fn puzzle(_input: &str) -> DayOutput {
    let errors: Vec<String> = Vec::new();
    let tabs: Vec<Tab> = Vec::new();
    DayOutput {
        silver_output: format!("{}", 0),
        gold_output: format!("{}", 0),
        diagnostic: Diagnostic::with_tabs(tabs, format!("{:?}", errors)),
    }
}
