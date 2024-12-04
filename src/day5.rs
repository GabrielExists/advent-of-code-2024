use std::iter::zip;
use regex::{Regex};
use crate::app::DayOutput;

pub fn puzzle(input: &str) -> DayOutput {
    let re = Regex::new(r"^([0-9]*)   ([0-9]*)$").unwrap();

    let dummy = 0 ;
    DayOutput {
        silver_output: format!("{}", dummy),
        gold_output: format!("{}", dummy),
        diagnostic: format!(""),
    }
}