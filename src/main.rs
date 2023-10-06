fn find_ranges(json: &str, target_keys: &[&str]) -> Vec<[usize; 2]> {
    let mut ranges = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    let mut in_string = false;
    let mut start_idx: Option<usize> = None;
    let mut skip_char = false;
    let mut is_key = false;
    let mut brace_count = 0;
    let mut bracket_count = 0;
    let mut capture_all = false;

    for (i, c) in json.chars().enumerate() {
        if skip_char {
            skip_char = false;
            continue;
        }

        match c {
            '{' => {
                if !in_string {
                    if stack == target_keys {
                        capture_all = true;
                        start_idx = Some(i);
                    }
                    brace_count += 1;
                }
                ranges.push([i, i + 1]);
            }
            '}' => {
                if !in_string {
                    brace_count -= 1;
                    if capture_all && brace_count == 0 {
                        capture_all = false;
                    }
                }
                ranges.push([i, i + 1]);
            }
            '[' => {
                if !in_string {
                    if stack == target_keys {
                        capture_all = true;
                        start_idx = Some(i);
                    }
                    bracket_count += 1;
                }
                ranges.push([i, i + 1]);
            }
            ']' => {
                if !in_string {
                    bracket_count -= 1;
                    if capture_all && bracket_count == 0 {
                        capture_all = false;
                    }
                }
                ranges.push([i, i + 1]);
            }
            ':' => {
                if !in_string {
                    ranges.push([i, i + 1]);
                    is_key = false;
                    // Capture the space after the colon
                    if json[i + 1..]
                        .chars()
                        .next()
                        .unwrap_or_default()
                        .is_whitespace()
                    {
                        let space_length = json[i + 1..]
                            .chars()
                            .take_while(|&ch| ch.is_whitespace())
                            .count();
                        ranges.push([i + 1, i + 1 + space_length]);
                    }
                }
            }
            '"' => {
                if capture_all {
                    if in_string {
                        ranges.push([start_idx.unwrap(), i + 1]);
                    } else {
                        start_idx = Some(i);
                    }
                    in_string = !in_string;
                } else {
                    if in_string {
                        // End of string
                        if let Some(start) = start_idx {
                            if stack == target_keys || is_key {
                                ranges.push([start, i + 1]);
                            }
                            start_idx = None;
                        }
                    } else {
                        // Start of string
                        start_idx = Some(i);
                        if let Some(next_double_quote) = json[i + 1..].find('"') {
                            let next_double_quote = next_double_quote + i + 1;
                            let content = &json[i + 1..next_double_quote];
                            if json[next_double_quote + 1..]
                                .chars()
                                .next()
                                .unwrap_or_default()
                                .is_whitespace()
                            {
                                let next_relevant_char = json[next_double_quote + 1..]
                                    .chars()
                                    .skip_while(|&ch| ch.is_whitespace())
                                    .next()
                                    .unwrap_or_default();
                                is_key = next_relevant_char == ':';
                            } else {
                                is_key = json[next_double_quote + 1..]
                                    .chars()
                                    .next()
                                    .unwrap_or_default()
                                    == ':';
                            }
                            if is_key {
                                stack.push(content.to_string());
                            }
                            skip_char = true;
                        } else {
                            // If there is no closing double quote, end the loop to prevent invalid behavior
                            break;
                        }
                    }
                    in_string = !in_string;
                }
            }
            ',' => {
                if capture_all {
                    // Capture the comma and the space after it if present
                    if let Some(next_char) = json.chars().nth(i + 1) {
                        if next_char.is_whitespace() {
                            ranges.push([i, i + 2]);
                            skip_char = true;
                        } else {
                            ranges.push([i, i + 1]);
                        }
                    }
                } else {
                    ranges.push([i, i + 1]);
                    if !in_string {
                        stack.pop();
                    }
                }
            }

            // For non-string values
            ch if ch.is_numeric() || ch == '-' || ch == '.' => {
                if capture_all {
                    if start_idx.is_none() {
                        start_idx = Some(i);
                    }
                    // If it's the end of a non-string value, push it to ranges
                    if let Some(next_char) = json[i + 1..].chars().next() {
                        if next_char == ','
                            || next_char == '}'
                            || next_char == ']'
                            || next_char.is_whitespace()
                        {
                            if let Some(start) = start_idx {
                                ranges.push([start, i + 1]);
                                start_idx = None;
                            }
                        }
                    }
                } else {
                    if start_idx.is_none() && stack == target_keys {
                        start_idx = Some(i);
                    }
                    // If it's the end of a non-string value and it matches the target keys, push it to ranges
                    if let Some(next_char) = json[i + 1..].chars().next() {
                        if next_char == ','
                            || next_char == '}'
                            || next_char == ']'
                            || next_char.is_whitespace()
                        {
                            if let Some(start) = start_idx {
                                ranges.push([start, i + 1]);
                                start_idx = None;
                            }
                        }
                    }
                }
            }
            _ => {
                if !in_string && !capture_all {
                    if start_idx.is_none() {
                        start_idx = Some(i);
                    }
                    // If it's the end of a non-string value, push it to ranges
                    if let Some(next_char) = json[i + 1..].chars().next() {
                        if next_char == ','
                            || next_char == '}'
                            || next_char == ']'
                            || next_char.is_whitespace()
                        {
                            if let Some(start) = start_idx {
                                if stack == target_keys {
                                    ranges.push([start, i + 1]);
                                }
                                start_idx = None;
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(start) = start_idx {
        ranges.push([start, json.len()]);
    }

    // Merge consecutive ranges
    ranges.sort_by(|a, b| a[0].cmp(&b[0]));
    let mut merged_ranges: Vec<[usize; 2]> = Vec::new();
    for range in ranges {
        if let Some(last_range) = merged_ranges.last_mut() {
            if last_range[1] >= range[0] {
                last_range[1] = range[1].max(last_range[1]);
            } else {
                merged_ranges.push(range);
            }
        } else {
            merged_ranges.push(range);
        }
    }

    merged_ranges
}

fn main() {
    let json = r#"{"a": {"b":"c","d":1,"e":[{"f":"g"}]}}"#;
    let keys = ["a", "e"];
    let ranges = find_ranges(json, &keys);
    println!("{:?}", ranges); // Expected: [[0, 11], [14, 19], [20, 38]]
}
