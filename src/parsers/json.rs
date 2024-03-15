use std::{
    collections::HashMap,
    num::{ParseFloatError, ParseIntError},
};

#[derive(Debug)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Float(f32),
    Int(i32),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, Box<JsonValue>>),
}

impl ToString for JsonValue {
    fn to_string(&self) -> String {
        match &self {
            Self::Null => "null".to_string(),
            Self::Bool(true) => "true".to_string(),
            Self::Bool(false) => "false".to_string(),
            Self::Int(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
            Self::String(value) => format!("\"{value}\""),
            Self::Object(map) => {
                let mut property_list = map
                    .iter()
                    .map(|(key, value)| format!("\"{key}\": {}", value.to_string()))
                    .collect::<Vec<String>>();
                property_list.sort();
                format!("{{\r\n{}\r\n}}", property_list.join(",\r\n"))
            }
            Self::Array(list) => {
                format!(
                    "[{}]",
                    list.iter()
                        .map(|value| value.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
        }
    }
}

impl PartialEq for JsonValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(l), Self::Bool(r)) => l == r,
            (Self::Int(l), Self::Int(r)) => l == r,
            (Self::Float(l), Self::Float(r)) => l == r,
            (Self::String(l), Self::String(r)) => l == r,
            (Self::Array(l0), Self::Array(r0)) => l0
                .iter()
                .zip(r0.iter())
                .fold(true, |equals, (l, r)| equals && (l == r)),
            (Self::Object(left), Self::Object(right)) => {
                left.iter().fold(true, |equals, (key, left_value)| {
                    let right_value = match right.get(key) {
                        Some(value) => value,
                        None => return false,
                    };
                    equals && (left_value == right_value)
                }) && right.iter().fold(true, |equals, (key, right_value)| {
                    let left_value = match left.get(key) {
                        Some(value) => value,
                        None => return false,
                    };
                    equals && (left_value == right_value)
                })
            }
            _ => false,
        }
    }
}

pub fn parse_json(json_str: &str) -> Option<JsonValue> {
    // go through the json
    let trimmed_str = json_str.trim();
    let int_value: Result<i32, ParseIntError> = trimmed_str.parse();
    let float_value: Result<f32, ParseFloatError> = trimmed_str.parse();
    let null_value = trimmed_str == "null";
    let boolean_value = trimmed_str == "true" || trimmed_str == "false";

    match trimmed_str {
        value if value.len() == 0 => None,

        value if value.starts_with("{") && value.ends_with("}") => parse_object(value),
        value if value.starts_with("[") && value.ends_with("]") => parse_array(value),
        value if value.starts_with("\"") && value.ends_with("\"") => Some(JsonValue::String(
            trimmed_str.to_string().replace(r#"""#, ""),
        )),

        _ if int_value.is_ok() => Some(JsonValue::Int(int_value.unwrap())),
        _ if float_value.is_ok() => Some(JsonValue::Float(float_value.unwrap())),
        _ if null_value => Some(JsonValue::Null),
        _ if boolean_value => Some(JsonValue::Bool(trimmed_str == "true")),
        _ => None,
    }
}

fn split_json(json_str: &str) -> Option<Vec<&str>> {
    let mut property_lines: Vec<&str> = Vec::new();
    let mut symbol_stack: Vec<char> = Vec::new();
    let mut start_idx: usize = 0;
    let mut end_idx: usize = 0;
    for (i, char) in json_str.chars().enumerate() {
        match char {
            '{' => symbol_stack.push('{'),
            '}' if (*symbol_stack.last().unwrap_or(&'f') == '{') => {
                symbol_stack.pop();
            }
            '}' => return None,
            '[' => symbol_stack.push('['),
            ']' if (*symbol_stack.last().unwrap_or(&'f') == '[') => {
                symbol_stack.pop();
            }
            ']' => return None,
            ',' if symbol_stack.len() == 0 => {
                property_lines.push(&json_str[start_idx..end_idx]);
                start_idx = i + 1;
            }
            _ => (),
        }
        end_idx = i + 1;
    }

    if symbol_stack.len() > 0 {
        return None;
    }

    if start_idx != end_idx {
        property_lines.push(&json_str[start_idx..end_idx])
    }

    Some(property_lines)
}

fn parse_object(object_str: &str) -> Option<JsonValue> {
    if object_str.len() < 2 || !object_str.starts_with("{") || !object_str.ends_with("}") {
        return None;
    };
    let trimmed_object_str = &object_str[1..object_str.len() - 1];
    let property_lines = match split_json(trimmed_object_str) {
        Some(v) => v,
        None => return None,
    };

    let property_map: HashMap<String, Box<JsonValue>> = property_lines
        .iter()
        .filter_map(|line| {
            let (key, value) = match line.split_once(":") {
                Some((key, value)) => (key, value),
                None => return None,
            };
            let json_value = parse_json(value.trim());
            match (key, json_value) {
                (key, Some(value)) => {
                    Some((key.trim().to_string().replace(r#"""#, ""), Box::new(value)))
                }
                _ => None,
            }
        })
        .collect();

    Some(JsonValue::Object(property_map))
}

fn parse_array(array_str: &str) -> Option<JsonValue> {
    if array_str.len() < 2 || !array_str.starts_with("[") || !array_str.ends_with("]") {
        return None;
    };
    let trimmed_array_str = &array_str[1..array_str.len() - 1];
    let values = match split_json(trimmed_array_str) {
        Some(v) => v,
        None => return None,
    };
    let json_values: Vec<JsonValue> = values
        .iter()
        .filter_map(|value| parse_json(value))
        .collect();

    Some(JsonValue::Array(json_values))
}

#[cfg(test)]
mod tests {
    mod parse_json {
        use super::super::*;
        #[test]
        fn empty_string() {
            assert!(parse_json("").is_none())
        }

        #[test]
        fn empty_object() {
            assert_eq!(parse_json("{}").unwrap(), JsonValue::Object(HashMap::new()))
        }

        #[test]
        fn empty_list() {
            assert_eq!(parse_json("[]").unwrap(), JsonValue::Array(vec![]))
        }

        #[test]
        fn one_dimensional_object() {
            assert_eq!(
                parse_json(r#"{"a": null}"#).unwrap(),
                JsonValue::Object(HashMap::from([(
                    "a".to_string(),
                    Box::new(JsonValue::Null)
                )]))
            )
        }

        #[test]
        fn two_dimensional_object() {
            assert_eq!(
                parse_json(r#"{"a": {"b": null}}"#).unwrap(),
                JsonValue::Object(HashMap::from([(
                    "a".to_string(),
                    Box::new(JsonValue::Object(HashMap::from([(
                        "b".to_string(),
                        Box::new(JsonValue::Null)
                    )])))
                )]))
            )
        }

        #[test]
        fn multiple_types_object() {
            assert_eq!(
                parse_json(r#"{"a": null, "b": 7, "c": "hello", "d": ["my life"], "e": false}"#)
                    .unwrap(),
                JsonValue::Object(HashMap::from([
                    ("a".to_string(), Box::new(JsonValue::Null)),
                    ("b".to_string(), Box::new(JsonValue::Int(7))),
                    (
                        "c".to_string(),
                        Box::new(JsonValue::String("hello".to_string()))
                    ),
                    (
                        "d".to_string(),
                        Box::new(JsonValue::Array(vec![JsonValue::String(
                            "my life".to_string()
                        )]))
                    ),
                    ("e".to_string(), Box::new(JsonValue::Bool(false)))
                ]))
            )
        }

        #[test]
        fn multiple_types_list() {
            assert_eq!(
                parse_json("[\"test string\", 10, 10.0, null, true]").unwrap(),
                JsonValue::Array(vec![
                    JsonValue::String("test string".to_string()),
                    JsonValue::Int(10),
                    JsonValue::Float(10.0),
                    JsonValue::Null,
                    JsonValue::Bool(true)
                ])
            )
        }

        #[test]
        fn bad_formatting() {
            assert_eq!(parse_json("{}}}"), None)
        }
    }

    mod json_str {
        use super::super::*;

        #[test]
        fn primitive() {
            assert_eq!("\"hi\"", JsonValue::String("hi".to_string()).to_string());
            assert_eq!("1", JsonValue::Int(1).to_string());
            assert_eq!("1.1", JsonValue::Float(1.1).to_string());
            assert_eq!("null", JsonValue::Null.to_string());
            assert_eq!("true", JsonValue::Bool(true).to_string());
            assert_eq!("false", JsonValue::Bool(false).to_string());
        }

        #[test]
        fn one_dimensional_object() {
            assert_eq!(
                "{\r\n\"a\": null,\r\n\"b\": 7,\r\n\"c\": \"hello\",\r\n\"d\": [\"my life\"],\r\n\"e\": false\r\n}",
                JsonValue::Object(HashMap::from([
                    ("a".to_string(), Box::new(JsonValue::Null)),
                    ("b".to_string(), Box::new(JsonValue::Int(7))),
                    (
                        "c".to_string(),
                        Box::new(JsonValue::String("hello".to_string()))
                    ),
                    (
                        "d".to_string(),
                        Box::new(JsonValue::Array(vec![JsonValue::String(
                            "my life".to_string()
                        )]))
                    ),
                    ("e".to_string(), Box::new(JsonValue::Bool(false)))
                ]))
                .to_string()
            )
        }

        #[test]
        fn one_dimensional_array() {
            assert_eq!(
                "[\"test string\", 10, 10.1, null, true]",
                JsonValue::Array(vec![
                    JsonValue::String("test string".to_string()),
                    JsonValue::Int(10),
                    JsonValue::Float(10.1),
                    JsonValue::Null,
                    JsonValue::Bool(true)
                ])
                .to_string()
            )
        }

        #[test]
        fn two_dimensional_object() {
            assert_eq!(
                "{\r\n\"a\": null,\r\n\"b\": 7,\r\n\"c\": \"hello\",\r\n\"d\": [\"my life\"],\r\n\"e\": false,\r\n\"f\": {\r\n\"g\": null\r\n}\r\n}",
                JsonValue::Object(HashMap::from([
                    ("a".to_string(), Box::new(JsonValue::Null)),
                    ("b".to_string(), Box::new(JsonValue::Int(7))),
                    (
                        "c".to_string(),
                        Box::new(JsonValue::String("hello".to_string()))
                    ),
                    (
                        "d".to_string(),
                        Box::new(JsonValue::Array(vec![JsonValue::String(
                            "my life".to_string()
                        )]))
                    ),
                    ("e".to_string(), Box::new(JsonValue::Bool(false))),
                    (
                        "f".to_string(),
                        Box::new(JsonValue::Object(HashMap::from([(
                            "g".to_string(),
                            Box::new(JsonValue::Null)
                        )])))
                    )
                ]))
                .to_string()
            )
        }
    }
}
