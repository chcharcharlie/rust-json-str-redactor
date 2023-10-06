fn find_ranges(json: &str, target_keys: &[&str]) -> Vec<[usize; 2]> {
    let mut in_string = false;
    let mut skip_char = false;
    let mut start_idx: Option<usize> = None;
    let mut brace_count = 0;
    let mut bracket_count = 0;
    let mut capture_all = false;
    let mut stack: Vec<&str> = Vec::new();
    let mut ranges: Vec<[usize; 2]> = Vec::new();

    for (i, c) in json.chars().enumerate() {
        if skip_char {
            skip_char = false;
            continue;
        }

        match c {
            '{' => {
                if stack == target_keys {
                    capture_all = true;
                    start_idx = Some(i);
                }
                brace_count += 1;
                ranges.push([i, i + 1]);
            }
            '}' => {
                brace_count -= 1;
                if capture_all && brace_count == 0 {
                    capture_all = false;
                }
                ranges.push([i, i + 1]);
                if !in_string {
                    stack.pop();
                }
            }
            '[' => {
                if stack == target_keys {
                    capture_all = true;
                    start_idx = Some(i);
                }
                bracket_count += 1;
                ranges.push([i, i + 1]);
            }
            ']' => {
                bracket_count -= 1;
                if capture_all && bracket_count == 0 {
                    capture_all = false;
                }
                ranges.push([i, i + 1]);
                if !in_string {
                    stack.pop();
                }
            }
            '"' => {
                if capture_all {
                    if in_string {
                        ranges.push([start_idx.unwrap(), i + 1]);
                        start_idx = None;
                    } else {
                        start_idx = Some(i);
                    }
                    in_string = !in_string;
                } else {
                    if in_string {
                        let key_or_value = &json[start_idx.unwrap() + 1..i];
                        if stack.last() == Some(&key_or_value) {
                            stack.pop();
                        } else {
                            stack.push(key_or_value);
                        }
                        ranges.push([start_idx.unwrap(), i + 1]);
                            start_idx = None;
                            start_idx = None;
                        }
                        start_idx = None;
                        }
                    } else {
                        start_idx = Some(i);
                    }
                    in_string = !in_string;
                }
            }
            ',' => {
                if capture_all {
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
            ch if ch.is_numeric() || ch == '-' || ch == '.' => {
                if capture_all {
                    if start_idx.is_none() {
                        start_idx = Some(i);
                    }
                } else {
                    if start_idx.is_none() && stack == target_keys {
                        start_idx = Some(i);
                    }
                }
            }
            _ => {
                if capture_all && start_idx.is_some() {
                    ranges.push([start_idx.unwrap(), i]);
                    start_idx = None;
                }
            }
        }
    }

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
