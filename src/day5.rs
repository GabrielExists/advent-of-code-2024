use crate::app::DayOutput;

pub fn puzzle(input: &str) -> DayOutput {
    let mut split = input.split("\n\n");
    if let (Some(input_pairs), Some(input_updates)) = (split.next(), split.next()) {
        // let re_pairs = Regex::new(r"(\d*)|(\d*)").unwrap();
        // let re_updates = Regex::new(r"((\d*),)*\n").unwrap();
        //
        // let updates = re_updates.captures_iter(input_updates).collect::<Vec<_>>();
        let rule_pairs = input_pairs.split("\n")
            .filter_map(|line| {
                let mut split = line.split("|");
                let first = split.next();
                let second = split.next();
                double_parse(first, second)
            }).collect::<Vec<(u64, u64)>>();

        let updates = input_updates.split("\n")
            .map(|line| {
                line.split(",")
                    .filter_map(|page| {
                        page.parse::<u64>().ok()
                    })
                    .collect::<Vec<u64>>()
            })
            .collect::<Vec<Vec<u64>>>();


        let mut sum_of_correct_middle_pages = 0;
        let mut sum_of_incorrect_middle_pages = 0;
        let mut errors = Vec::new();
        for update in updates.iter() {
            let mut ok = true;
            for (index, page) in update.iter().enumerate() {
                for latter_page in &update[index + 1..] {
                    for (rule_first, rule_second) in rule_pairs.iter() {
                        // If we have a rule disallowing this pair
                        if *latter_page == *rule_first && *page == *rule_second {
                            ok = false;
                        }
                    }
                }
            }
            if ok {
                if let Some(page) = update.get( update.len() / 2 ) {
                    sum_of_correct_middle_pages += page;
                }
            } else {
                let mut update = update.clone();
                let mut rule_pairs = rule_pairs.clone();

                let mut new_update = Vec::new();
                while !update.is_empty() {
                    rule_pairs = rule_pairs.into_iter().filter(|(rule_first, rule_second)| {
                        update.contains(rule_first) && update.contains(rule_second)
                    }).collect::<Vec<(u64, u64)>>();

                    let mut candidates = update.clone();
                    // Find one or more candidates that have nothing that needs to go before it
                    for (_rule_first, rule_second) in rule_pairs.iter() {
                        candidates = candidates.into_iter()
                            .filter(|candidate| candidate != rule_second)
                            .collect::<Vec<_>>()
                    }
                    if candidates.is_empty() {
                        // We're stuck. Abort and error
                        errors.push(format!("Error with {:?} added and {:?} remaining", new_update, update));
                        break;
                    }
                    // Add all candidates to the new update
                    for candidate in candidates.into_iter() {
                        update.retain(|page| *page != candidate);
                        new_update.push(candidate);
                    }
                }
                if let Some(page) = new_update.get( new_update.len() / 2 ) {
                    sum_of_incorrect_middle_pages += page;
                }
            }
        }

        DayOutput {
            silver_output: format!("{}", sum_of_correct_middle_pages),
            gold_output: format!("{}", sum_of_incorrect_middle_pages),
            diagnostic: format!("{:?}", errors),
        }
    } else {
        DayOutput {
            silver_output: "".to_string(),
            gold_output: "".to_string(),
            diagnostic: "Failed to split".to_string(),
        }
    }
}

pub fn double_parse(first: Option<&str>, second: Option<&str>) -> Option<(u64, u64)> {
    match (
        first.map(|item| item.parse::<u64>()),
        second.map(|item| item.parse::<u64>())
    ) {
        (
            Some(Ok(first)),
            Some(Ok(second))
        ) => Some((first, second)),
        _ => None,
    }
}
