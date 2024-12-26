#![allow(dead_code)]

use std::str::FromStr;
use regex::Captures;

pub fn capture_parse<F: FromStr>(captures: &Captures, name: &str) -> Option<F> {
    captures.name(name).map(|s| s.as_str().parse::<F>().ok()).unwrap_or(None)
}

pub fn combine_2<T1, T2>(t1: Option<T1>, t2: Option<T2>) -> Option<(T1, T2)> {
    match (t1, t2) {
        (Some(t1), Some(t2)) => {
            Some((t1, t2))
        }
        _ => None,
    }
}

pub fn combine_3<T1, T2, T3>(t1: Option<T1>, t2: Option<T2>, t3: Option<T3>) -> Option<(T1, T2, T3)> {
    match (t1, t2, t3) {
        (Some(t1), Some(t2), Some(t3)) => {
            Some((t1, t2, t3))
        }
        _ => None,
    }
}

pub fn combine_4<T1, T2, T3, T4>(t1: Option<T1>, t2: Option<T2>, t3: Option<T3>, t4: Option<T4>) -> Option<(T1, T2, T3, T4)> {
    match (t1, t2, t3, t4) {
        (Some(t1), Some(t2), Some(t3), Some(t4)) => {
            Some((t1, t2, t3, t4))
        }
        _ => None,
    }
}

pub fn combine_5<T1, T2, T3, T4, T5>(t1: Option<T1>, t2: Option<T2>, t3: Option<T3>, t4: Option<T4>, t5: Option<T5>) -> Option<(T1, T2, T3, T4, T5)> {
    match (t1, t2, t3, t4, t5) {
        (Some(t1), Some(t2), Some(t3), Some(t4), Some(t5)) => {
            Some((t1, t2, t3, t4, t5))
        }
        _ => None,
    }
}
