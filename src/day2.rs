use regex::Regex;
use Direction::*;
use crate::app::{DayOutput, Diagnostic};

trait SkipSpec<S>: Iterator + Sized {
    fn skip_specific(self, skip_index: S) -> SkipSpecific<Self, S>;
}

struct SkipSpecific<T, S> {
    iter: T,
    skip_indices: S,
    current_index: usize,
}

impl<T, S> SkipSpec<S> for T
where T: Iterator, S: SkipIndices {
    fn skip_specific(self, skip_indices: S) -> SkipSpecific<Self, S> {
        SkipSpecific {
            iter: self,
            skip_indices,
            current_index: 0,
        }
    }
}

impl<T, S> Iterator for SkipSpecific<T, S>
where T: Iterator, S: SkipIndices {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {

        while self.skip_indices.contains_index(self.current_index) {
            self.iter.next()?;
            self.current_index += 1;
        }
        self.current_index += 1;
        self.iter.next()
    }
}

trait SkipIndices {
    fn contains_index(&self, index: usize) -> bool;
}

impl SkipIndices for usize {
    fn contains_index(&self, index: usize) -> bool {
        *self == index
    }
}
impl SkipIndices for &[usize] {
    fn contains_index(&self, index: usize) -> bool {
        self.iter().any(|item| *item == index)
    }
}

pub fn puzzle(input: &str) -> DayOutput {
    let re = Regex::new(r"([0-9]*) ").unwrap();
    let mut num_matches = 0;
    let reports = format!("{} ", input).split("\n")
        .filter_map(|line| {
            num_matches += 1;
            let report = re.find_iter(&format!("{} ", line))
                .filter_map(|current_match| {
                    match current_match.as_str().trim().parse::<u64>() {
                        Ok(level) => {
                            Some(level)
                        }
                        _ => {
                            None
                        }
                    }
                })
                .collect::<Vec<_>>();
            if report.is_empty() {
                None
            } else {
                Some(report)
            }
        })
        .collect::<Vec<_>>();

    let report_length = reports.len();
    let mut safe_reports = 0;
    let mut safe_reports_allowing_one = 0;
    let mut error_message = None;
    for (_index, report) in reports.into_iter().enumerate() {
        match judge_safety(report.iter().copied()) {
            Ok(()) => {
                safe_reports += 1;
                safe_reports_allowing_one += 1;
            }
            Err(error) => {
                if let None = error_message {
                    error_message = Some(error);
                }
                for i in 0..report.len() {
                    let mut current_report = report.clone();
                    current_report.remove(i);
                    // match judge_safety(current_report.iter().copied()) {
                    match judge_safety(report.iter().skip_specific(&[i][..]).copied()) {
                        Ok(()) => {
                            safe_reports_allowing_one += 1;
                            break;
                        }
                        Err(_) => {}
                    }
                }
            }
        }
    }
    DayOutput {
        silver_output: format!("{}", safe_reports),
        gold_output: format!("{:?}", safe_reports_allowing_one),
        diagnostic: Diagnostic::simple(format!("Num reports: {}, first error message \n{:?}", report_length, error_message)),
    }
}

enum Direction {
    Undetermined,
    Increasing,
    Decreasing,
}

fn judge_safety<T: IntoIterator<Item=u64>>(report: T) -> Result<(), String>
where T: IntoIterator<Item=u64>{
    let mut stored_previous = None;
    let mut direction = Undetermined;
    for (index, level) in report.into_iter().enumerate() {
        match stored_previous {
            None => {
                stored_previous = Some(level);
            }
            Some(previous_level) => {
                if previous_level > level {
                    if previous_level - level <= 3 {
                        match direction {
                            Undetermined => {
                                direction = Decreasing;
                            }
                            Increasing => {
                                return Err(format!("Direction changed from increasing to decreasing at index {}.", index));
                            }
                            Decreasing => {}
                        }
                    } else {
                        return Err(format!("Decremented with more than 3, {} to {}, index {}", previous_level, level, index));
                    }
                } else if level > previous_level {
                    if level - previous_level <= 3 {
                        match direction {
                            Undetermined => {
                                direction = Increasing;
                            }
                            Increasing => {}
                            Decreasing => {
                                return Err(format!("Direction changed from decreasing to increasing at index {}.", index));
                            }
                        }
                    } else {
                        return Err(format!("Incremented with more than 3, {} to {}, index {}", level, previous_level, index));
                    }
                } else { //equal
                    return Err(format!("Two consecutive equal values at index {}", index));
                }
                stored_previous = Some(level);
            }
        }
    }
    Ok(())
}