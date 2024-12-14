use regex::{Regex};
use crate::app::{DayOutput, Diagnostic, Tab};

#[derive(Clone, Debug)]
struct Problem {
    ax: u64,
    ay: u64,
    bx: u64,
    by: u64,
    px: u64,
    py: u64,
}

const A_COST: u64 = 3;
const B_COST: u64 = 1;
const GOLD_ADJUSTMENT: u64 = 10000000000000;

pub fn puzzle(input: &str) -> DayOutput {
    let re = Regex::new(r"Button A: X\+(?P<AX>\d*), Y\+(?P<AY>\d*)\nButton B: X\+(?P<BX>\d*), Y\+(?P<BY>\d*)\nPrize: X\=(?P<PX>\d*), Y\=(?P<PY>\d*)").unwrap();
    let problems: Vec<Problem> = re.captures_iter(input).into_iter().filter_map(|captures| {
        let ax = captures.name("AX").map(|s| s.as_str().parse::<u64>().ok()).unwrap_or(None);
        let ay = captures.name("AY").map(|s| s.as_str().parse::<u64>().ok()).unwrap_or(None);
        let bx = captures.name("BX").map(|s| s.as_str().parse::<u64>().ok()).unwrap_or(None);
        let by = captures.name("BY").map(|s| s.as_str().parse::<u64>().ok()).unwrap_or(None);
        let px = captures.name("PX").map(|s| s.as_str().parse::<u64>().ok()).unwrap_or(None);
        let py = captures.name("PY").map(|s| s.as_str().parse::<u64>().ok()).unwrap_or(None);
        if let (Some(ax), Some(ay), Some(bx), Some(by), Some(px), Some(py)) = (ax, ay, bx, by, px, py) {
            Some(Problem {
                ax,
                ay,
                bx,
                by,
                px,
                py,
            })
        } else {
            None
        }
    }).collect::<Vec<_>>();

    let mut status = Vec::new();
    let mut status_gold = Vec::new();
    let mut total_coins = 0;
    let mut total_coins_gold = 0;
    for problem in problems.iter() {
        if let Some(coins) = solve(problem, 0, 0, &mut status) {
            total_coins += coins;
        }
        let mut problem = problem.clone();
        problem.px += GOLD_ADJUSTMENT;
        problem.py += GOLD_ADJUSTMENT;
        if let Some(coins) = solve_gold(&problem, &mut status_gold) {
            total_coins_gold += coins;
        }
    }

    let mut tabs = Vec::new();
    tabs.push(Tab {
        title: "Input".to_string(),
        strings: problems.iter().map(|problem| format!("Problem: a: {}, {}, b: {}, {}, p: {}, {}", problem.ax, problem.ay, problem.bx, problem.by, problem.px, problem.py)).collect(),
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Status".to_string(),
        strings: status,
        grid: vec![],
    });
    tabs.push(Tab {
        title: "Status gold".to_string(),
        strings: status_gold,
        grid: vec![],
    });
    DayOutput {
        silver_output: format!("{}", total_coins),
        gold_output: format!("{}", total_coins_gold),
        diagnostic: Diagnostic::with_tabs(tabs, format!("")),
    }
}

fn solve(problem: &Problem, a_presses: u64, b_presses: u64, status: &mut Vec<String>) -> Option<u64> {
    let x = problem.ax * a_presses + problem.bx * b_presses;
    let y = problem.ay * a_presses + problem.by * b_presses;
    if problem.px == x && problem.py == y {
        status.push(format!("Found at a: {}, b: {}", a_presses, b_presses));
        Some(a_presses * A_COST + b_presses * B_COST)
    } else {
        // status.push(format!("a: {}, b: {}", a_presses, b_presses));
        if a_presses > 0 {
            if a_presses <= 100 {
                solve(problem, a_presses + 1, b_presses, status)
            } else {
                None
            }
        } else {
            let a_path = if a_presses <= 100 {
                solve(problem, a_presses + 1, b_presses, status)
            } else {
                None
            };
            let b_path = if b_presses <= 100 {
                solve(problem, a_presses, b_presses + 1, status)
            } else {
                None
            };
            match (a_path, b_path) {
                (None, None) => None,
                (None, Some(b_path)) => Some(b_path),
                (Some(a_path), None) => Some(a_path),
                (Some(a_path), Some(b_path)) => {
                    Some(std::cmp::min(a_path, b_path))
                },
            }
        }
    }
}

fn solve_gold(problem: &Problem, status: &mut Vec<String>) -> Option<u64> {
    let ax = problem.ax as f64;
    let ay = problem.ay as f64;
    let bx = problem.bx as f64;
    let by = problem.by as f64;
    let px = problem.px as f64;
    let py = problem.py as f64;

    // ax * a + bx * b = px
    // ay * a + by * b = py
    // (px - bx * b) / ax = (py - by * b) / bx
    // px / ax - py / ay = b (bx / ax - by / ay)
    // b = (px / ax - py / ay) / (bx / ax - by / ay)
    // a = (px - bx * b) / ax
    let b = (px / ax - py / ay) / (bx / ax - by / ay);
    let a = (px - bx * b) / ax;

    status.push(format!("a{}, b{}", a, b));

    // These systems of linear equations are such that there's only one solution
    // If that solution happens to be an integer, then we ship it.
    match (try_to_int(a, status), try_to_int(b, status)) {
        (Some(a), Some(b)) => {
            status.push(format!("int a{}, b{}", a, b));
            Some(a * 3 + b)
        }
        (_, _) => {
            None
        }
    }
}

fn try_to_int(input: f64, status: &mut Vec<String>) -> Option<u64> {
    let int = input.round() as u64;
    let loopback = int as f64;
    status.push(format!("input: {}, int: {}, loopback: {}", input, int, loopback));
    let delta = f64::abs(input - loopback);
    if delta < 0.001 {
        Some(int)
    } else {
        None
    }
}