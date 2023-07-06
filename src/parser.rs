use pest;
// use pest::error::{Error};
use pest::iterators::Pair;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammer.pest"]
pub struct JsonParser;

// Helper function to parse a JSON string
fn parse_string(pair: Pair<'_, Rule>) -> String {
    let inner = pair.into_inner().next().unwrap();
    inner.as_str().trim_matches('"').to_owned()
}

// Helper function to parse a JSON number
fn parse_number(pair: Pair<'_, Rule>) -> serde_json::Number {
    serde_json::Number::from_f64(pair.as_str().parse().unwrap()).unwrap()
}

// Helper function to parse a JSON boolean
fn parse_boolean(pair: Pair<'_, Rule>) -> bool {
    match pair.as_str() {
        "true" => true,
        "false" => false,
        _ => unreachable!(),
    }
}

// // Helper function to parse a JSON null value
// fn parse_null(_: pest::iterators::Pair<'_, Rule>) -> Option<()> {
//     Some(())
// }

// Recursive function to parse a JSON value
pub fn parse_value(pair: Pair<'_, Rule>) -> serde_json::Value {
    println!("{}", pair.as_str());
    println!("{:?}", pair.as_rule());

    match pair.as_rule() {
        Rule::identifier => parse_function(pair), // Pass the inner pair to parse_function
        Rule::object => parse_object(pair),
        Rule::array => parse_array(pair),
        Rule::string => serde_json::Value::String(parse_string(pair)),
        Rule::number => serde_json::Value::Number(parse_number(pair)),
        Rule::boolean => serde_json::Value::Bool(parse_boolean(pair)),
        Rule::null => serde_json::Value::Null,
        _ => unreachable!(),
    }
}

// Helper function to parse a JSON object
fn parse_object(pair: Pair<'_, Rule>) -> serde_json::Value {
    println!("{}", pair.as_str());
    println!("{:?}", pair.as_rule());

    let mut object = serde_json::Map::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::pair => {
                let mut inner_pairs = inner.into_inner();
                let key = parse_string(inner_pairs.next().unwrap());
                let value = parse_value(inner_pairs.next().unwrap());
                object.insert(key, value);
            }
            _ => unreachable!(),
        }
    }
    serde_json::Value::Object(object)
}


// Helper function to parse a JSON array
fn parse_array(pair: Pair<'_, Rule>) -> serde_json::Value {
    let mut array = Vec::new();

    println!("{}", pair.as_str());
    println!("{:?}", pair.as_rule());

    for inner in pair.into_inner() {
        array.push(parse_value(inner));
    }
    serde_json::Value::Array(array)
}

// Helper function to parse a JSON function
fn parse_function(pair: Pair<'_, Rule>) -> serde_json::Value {
    
    let identifier = pair.as_str();
    println!("{:?}", pair.as_rule());
    println!("{:?}", pair.as_node_tag());
    println!("{:?}", pair.as_span());
    let mut inner_pairs = pair.into_inner();
    println!("{:?}", inner_pairs);
    let args = inner_pairs.next().map(|pair| parse_function_args(pair));
    
    // Handle function name resolution
    match identifier {
        // Example function name: string(length, prefix, suffix)
        "string" => {
            // Retrieve function arguments
            let arg_values = args.unwrap_or_default();
            let length = arg_values.get(0).and_then(|v| v.as_u64()).unwrap_or(0);
            let prefix = arg_values.get(1).and_then(|v| v.as_str()).unwrap_or("");
            let suffix = arg_values.get(2).and_then(|v| v.as_str()).unwrap_or("");

            // Generate the string based on the function arguments
            let generated_string = format!("{}{}{}", prefix, generate_random_string(length), suffix);

            // Return the generated string as a JSON string value
            serde_json::Value::String(generated_string)
        }
        // Add more function names and their corresponding actions here
        _ => {
            // Handle unrecognized function names
            serde_json::Value::String(format!("Unrecognized function: {}({:?})", identifier, args))
        }
    }
}

// Function to generate a random string of a given length
fn generate_random_string(length: u64) -> String {
    // Implementation of generating a random string goes here
    // ...
    "RandomStringPlaceholder".to_string()
}

// Helper function to parse function arguments
fn parse_function_args(pair: Pair<'_, Rule>) -> Vec<serde_json::Value> {
    pair.into_inner().map(parse_value).collect()
}