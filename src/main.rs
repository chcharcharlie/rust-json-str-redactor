fn find_ranges(json: &str, target_keys: &[&str]) -> Vec<[usize; 2]> {
    let mut ranges = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    let mut in_string = false;
    let mut start_idx: Option<usize> = None;
    let mut skip_char = false;
    let mut is_key = false;

    for (i, c) in json.chars().enumerate() {
        if skip_char {
            skip_char = false;
            continue;
        }

        match c {
            '{' | '}' | '[' | ']' | ',' | ':' => {
                // If we were processing a non-string value, capture its range
                if let Some(start) = start_idx {
                    if stack == target_keys {
                        ranges.push([start, i]);
                    }
                    start_idx = None;
                }
                ranges.push([i, i + 1]);
                if c == ':'
                    && json[i + 1..]
                        .chars()
                        .next()
                        .unwrap_or_default()
                        .is_whitespace()
                {
                    // Include the space after the colon
                    let space_length = json[i + 1..]
                        .chars()
                        .take_while(|&ch| ch.is_whitespace())
                        .count();
                    ranges.push([i + 1, i + 1 + space_length]);
                }
            }
            '"' => {
                if in_string {
                    // End of string
                    if let Some(start) = start_idx {
                        if stack == target_keys || is_key {
                            ranges.push([start, i + 1]);
                        }
                        start_idx = None;
                    }
                    if is_key {
                        stack.pop();
                    }
                } else {
                    // Start of string
                    start_idx = Some(i);
                    let next_double_quote = json[i + 1..].find('"').unwrap() + i + 1;
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
                }
                in_string = !in_string;
            }
            _ => {
                if !in_string {
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
            if last_range[1] == range[0] {
                last_range[1] = range[1];
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
    let keys = ["a", "d"];
    let ranges = find_ranges(json, &keys);
    println!("{:?}", ranges); // Expected: [[0, 11], [14, 31], [34, 38]]
}
