use std::str::FromStr;
use regex::Captures;

pub fn capture_parse<F: FromStr>(captures: &Captures, name: &str) -> Option<F> {
    captures.name(name).map(|s| s.as_str().parse::<F>().ok()).unwrap_or(None)
}

