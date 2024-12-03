use regex::Regex;
use Direction::*;

pub fn puzzle(input: &str) -> (String, String) {
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

    let _report_length = reports.len();
    let mut safe_reports = 0;
    let mut error_message = format!("{:?}", reports);
    for (index, report) in reports.into_iter().enumerate() {
        match judge_safety(report) {
            Ok(()) => {
                safe_reports += 1;
            }
            Err(error) => {
                // error_message = Some(format!("Index {}: {}", index, error));
            }
        }
    }
    (format!("{}", safe_reports), format!("{:?}", error_message))
}

enum Direction {
    Undetermined,
    Increasing,
    Decreasing,
}

fn judge_safety(report: Vec<u64>) -> Result<(), String> {
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