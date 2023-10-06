use serde_json::{Map, Value};

fn find_ranges(json: &str, keys: &[&str]) -> Vec<[usize; 2]> {
    let parsed: Value = serde_json::from_str(json).unwrap();
    let mut ranges = Vec::new();
    traverse(&parsed, keys, &mut ranges, 0, Vec::new());
    ranges
}

fn traverse(
    value: &Value,
    keys: &[&str],
    ranges: &mut Vec<[usize; 2]>,
    pos: usize,
    mut current_keys: Vec<&str>,
) -> usize {
    match value {
        Value::Object(map) => {
            let mut current_pos = pos + 1; // Opening brace '{'
            for (key, val) in map {
                let key_len = key.len() + 2; // For the quotes around the key
                ranges.push([current_pos, current_pos + key_len]); // Include the key range
                current_pos += key_len + 1; // Move past the key and the colon ':'

                current_keys.push(key);
                current_pos = traverse(val, keys, ranges, current_pos, current_keys.clone());
                current_keys.pop();

                if let Some(next_char) = map.get(&key) {
                    current_pos += match next_char {
                        Value::String(_) => 1,
                        _ => 0,
                    };
                }

                current_pos += 1; // Move past the comma ',' or closing brace '}'
            }
            current_pos
        }
        Value::Array(arr) => {
            let mut current_pos = pos + 1; // Opening bracket '['
            for val in arr {
                current_pos = traverse(val, keys, ranges, current_pos, current_keys.clone());
                current_pos += 1; // Move past the comma ',' or closing bracket ']'
            }
            current_pos
        }
        Value::String(s) => {
            let len = s.len() + 2; // For the quotes around the string
            if &current_keys[..] == keys {
                ranges.push([pos, pos + len]);
            }
            pos + len
        }
        _ => {
            let s = value.to_string();
            let len = s.len();
            if &current_keys[..] == keys {
                ranges.push([pos, pos + len]);
            }
            pos + len
        }
    }
}

fn main() {
    let json = r#"{"a": {"b":"c","d":1,"e":[{"f":"g"}]}}"#;
    let keys = ["a", "d"];
    let ranges = find_ranges(json, &keys);
    println!("{:?}", ranges);
}
