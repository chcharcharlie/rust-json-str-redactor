use serde_json::Value;
use std::collections::HashSet;

// Sample input:
// let json = r#"{"name":"Alice","age":30,"contacts":[{"type":"email","value":"alice@email.com"},{"type":"phone","value":"123-456-7890"}],"isActive":true}"#;
// let keys = [["contacts", "type"].to_vec(), ["name"].to_vec()];

// Sample output:
// [[0, 22], [24, 61], [78, 104], [118, 132], [136, 137]]
// {
//   "age": "<REDACTED>",
//   "contacts": [
//     {
//       "type": "email",
//       "value": "<REDACTED>"
//     },
//     {
//       "type": "phone",
//       "value": "<REDACTED>"
//     }
//   ],
//   "isActive": "<REDACTED>",
//   "name": "Alice"
// }

fn find_ranges(json: &str, target_keys_list: &[Vec<&str>]) -> Vec<[usize; 2]> {
    // Create a HashSet to store all the ranges
    let mut all_ranges = HashSet::new();

    for target_keys in target_keys_list {
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
                        stack.push("{".to_string());
                        if stack
                            .iter()
                            .filter(|&&ref s| s != "{" && s != "[")
                            .eq(target_keys.iter().cloned())
                        {
                            capture_all = true;
                            start_idx = Some(i);
                        }
                        brace_count += 1;
                        ranges.push([i, i + 1]);
                    } else if capture_all {
                        ranges.push([i, i + 1]);
                    }
                }
                '}' if !in_string => {
                    brace_count -= 1;
                    if capture_all && brace_count == 1 {
                        capture_all = false;
                    }
                    ranges.push([i, i + 1]);
                    // Pop the stack until we find the matching '{'
                    while let Some(top) = stack.pop() {
                        if &top == "{" {
                            break;
                        }
                    }
                    // If the top of the stack is a key (and not another '{' or '['), pop that key as well
                    if let Some(top) = stack.last() {
                        if top != "{" && top != "[" {
                            stack.pop();
                        }
                    }
                }
                '[' => {
                    if !in_string {
                        stack.push("[".to_string());
                        if stack
                            .iter()
                            .filter(|&&ref s| s != "{" && s != "[")
                            .eq(target_keys.iter().cloned())
                        {
                            capture_all = true;
                            start_idx = Some(i);
                        }
                        bracket_count += 1;
                        ranges.push([i, i + 1]);
                    } else if capture_all {
                        ranges.push([i, i + 1]);
                    }
                }
                ']' if !in_string => {
                    bracket_count -= 1;
                    if capture_all && bracket_count == 0 {
                        capture_all = false;
                    }
                    ranges.push([i, i + 1]);
                    // Pop the stack until we find the matching '['
                    while let Some(top) = stack.pop() {
                        if &top == "[" {
                            break;
                        }
                    }
                    // If the top of the stack is a key (and not another '[' or '{'), pop that key as well
                    if let Some(top) = stack.last() {
                        if top != "[" && top != "{" {
                            stack.pop();
                        }
                    }
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
                                if stack
                                    .iter()
                                    .filter(|&&ref s| s != "{" && s != "[")
                                    .eq(target_keys.iter().cloned())
                                    || is_key
                                {
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
                    } else if !in_string {
                        ranges.push([i, i + 1]);
                        // If the top of the stack is a key (and not another '[' or '{'), pop that key as well
                        if let Some(top) = stack.last() {
                            if top != "[" && top != "{" {
                                stack.pop();
                            }
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
                        if start_idx.is_none()
                            && stack
                                .iter()
                                .filter(|&&ref s| s != "{" && s != "[")
                                .eq(target_keys.iter().cloned())
                        {
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
                    if !in_string {
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
                                        if stack
                                            .iter()
                                            .filter(|&&ref s| s != "{" && s != "[")
                                            .eq(target_keys.iter().cloned())
                                        {
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
        }

        if let Some(start) = start_idx {
            ranges.push([start, json.len()]);
        }

        for range in &ranges {
            all_ranges.insert(*range);
        }
    }

    // Convert the HashSet back to a Vec
    let mut ranges: Vec<_> = all_ranges.into_iter().collect();

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

fn redact_json(json: &str, ranges: Vec<[usize; 2]>) -> String {
    let mut result = String::new();
    let mut last_idx = 0;

    for range in ranges {
        // Append the redacted portion if there's a gap between ranges
        if last_idx < range[0] {
            result.push_str("\"<REDACTED>\"");
        }

        // Append the preserved portion from the range
        result.push_str(&json[range[0]..range[1]]);
        last_idx = range[1];
    }

    // Handle any remaining content after the last range
    if last_idx < json.len() {
        result.push_str("\"<REDACTED>\"");
    }

    result
}

fn main() {
    let json = r#"{"name":"Alice","age":30,"contacts":[{"type":"email","value":"alice@email.com"},{"type":"phone","value":"123-456-7890"}],"isActive":true}"#;
    let keys = [["contacts", "type"].to_vec(), ["name"].to_vec()];
    let ranges = find_ranges(json, &keys);
    println!("{:?}", ranges);

    let result = redact_json(json, ranges);
    // Parse the resultant string into a serde_json::Value
    let parsed_value: Value = serde_json::from_str(&result).expect("Failed to parse JSON");
    // Pretty print the JSON
    let result =
        serde_json::to_string_pretty(&parsed_value).expect("Failed to generate pretty JSON");
    println!("{}", result);
}
