// use std::{error::Error, fmt::Display};

// #[derive(Debug)]
// pub struct JGError {
//     message: String
// }

// impl Display for JGError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.write_str(self.message.as_str())
//     }
// }

// impl Error for JGError {}

// pub fn generate_json(json_schema: String) -> Result<String, JGError> {
//     Ok(json_schema)
// }


use serde_json::Value;
use rand::Rng;

pub fn generate_json(json: &Value) -> Value {
    match json {
        Value::Object(obj) => {
            let mut generated_obj = serde_json::Map::new();
            for (key, value) in obj {
                let generated_value = generate_json(value);
                generated_obj.insert(key.clone(), generated_value);
            }
            Value::Object(generated_obj)
        }
        Value::Array(arr) => {
            let generated_arr: Vec<Value> = arr
                .iter()
                .map(|value| generate_json(value))
                .collect();
            Value::Array(generated_arr)
        }
        Value::String(s) => {
            if s.starts_with("string") {
                generate_random_string_from_function(s)
            } else {
                Value::String(s.clone())
            }
        }
        _ => json.clone(),
    }
}

fn generate_random_string_from_function(s: &str) -> Value {
    let args: Vec<&str> = s
        .trim_start_matches("string(")
        .trim_end_matches(')')
        .split(',')
        .map(|arg| arg.trim())
        .collect();

    if args.len() != 4 {
        eprintln!("Invalid arguments for string function");
        return Value::String(s.to_string());
    }

    let min_length = match args[0].parse::<usize>() {
        Ok(length) => length,
        Err(_) => {
            eprintln!("Invalid minLength argument for string function");
            return Value::String(s.to_string());
        }
    };

    let max_length = match args[1].parse::<usize>() {
        Ok(length) => length,
        Err(_) => {
            eprintln!("Invalid maxLength argument for string function");
            return Value::String(s.to_string());
        }
    };

    let prefix = args[2];
    let suffix = args[3];

    let random_string = generate_random_string(min_length, max_length);

    Value::String(format!("{}{}{}", prefix, random_string, suffix))
}

fn generate_random_string(min_length: usize, max_length: usize) -> String {
    let mut rng = rand::thread_rng();
    let length = rng.gen_range(min_length..=max_length);
    let random_string: String = (0..length)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect();
    random_string
}