fn find_ranges(json: &str, target_keys: &[&str]) -> Vec<[usize; 2]> {
    let mut ranges = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    let mut in_string = false;
    let mut start_idx: Option<usize> = None;
    let mut skip_char = false;

    for (i, c) in json.chars().enumerate() {
        if skip_char {
            skip_char = false;
            continue;
        }

        match c {
            '{' | '}' | '[' | ']' | ',' | ':' => {
                if !in_string {
                    if let Some(start) = start_idx {
                        if stack == target_keys {
                            ranges.push([start, i]);
                        }
                        start_idx = None;
                    }
                    ranges.push([i, i + 1]);
                }
            }
            '"' => {
                if in_string {
                    // End of string
                    stack.pop();
                    if let Some(start) = start_idx {
                        ranges.push([start, i + 1]);
                        start_idx = None;
                    }
                } else {
                    // Start of string
                    start_idx = Some(i);
                    let next_double_quote = json[i + 1..].find('"').unwrap() + i + 1;
                    let key = &json[i + 1..next_double_quote];
                    stack.push(key.to_string());
                    skip_char = true;
                }
                in_string = !in_string;
            }
            _ => {
                if !in_string {
                    if start_idx.is_none() {
                        start_idx = Some(i);
                    }
                    if c.is_whitespace() {
                        if let Some(start) = start_idx {
                            if stack == target_keys {
                                ranges.push([start, i]);
                            }
                            start_idx = None;
                        }
                    }
                }
            }
        }
    }

    if let Some(start) = start_idx {
        ranges.push([start, json.len()]);
    }

    ranges
}

fn main() {
    let json = r#"{"a": {"b":"c","d":1,"e":[{"f":"g"}]}}"#;
    let keys = ["a", "d"];
    let ranges = find_ranges(json, &keys);
    println!("{:?}", ranges); // Expected: [[0, 11], [14, 31], [34, 38]]
}
