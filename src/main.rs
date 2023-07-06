use std::fs;
use std::io::{self, Read};
use clap::{App, Arg};

use jg::generator;
use jg::parser::{self, Rule};
use pest::Parser;

fn main() {
    // Parse command line arguments using clap
    let  matches = App::new("JSON Generator")
        .arg(
            Arg::with_name("input")
                .value_name("INPUT")
                .help("Input file path")
                .required(false),
        )
        .get_matches();

    // Read the input file or stdin
    let json_schema = match matches.value_of("input") {
        Some(input_path) => {
            let mut file = fs::File::open(input_path).expect("Failed to open input file");
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .expect("Failed to read input file");
            contents
        }
        None => {
            // let mut input = String::new();
            // io::stdin().read_to_string(&mut input).expect("Failed to read from stdin");
            // input
            "{ \"myKey\": string(5,10, \"asd\", \"qwe\") }".to_string()
            //"{\"myKey\": [123.321, string(5,10, \"asd\", \"qwe\")]}".to_string()
        }
    };

    // // Generate the JSON object
    // match generator::generate_json(json_schema) {
    //     Ok(json) => println!("{}", json),
    //     Err(err) => eprintln!("Error: {}", err),
    // }

     // Parse JSON generation definitions
     let parsed_json = match parser::JsonParser::parse(Rule::json, &json_schema) {
        Ok(mut pairs) => {
            let pair = pairs.next().unwrap();
            parser::parse_value(pair)
        }
        Err(err) => {
            eprintln!("Error parsing JSON generation definitions: {}", err);
            return;
        }
    };


    // Generate JSON
    let generated_json = generator::generate_json(&parsed_json);

    // Print generated JSON
    println!("{}", serde_json::to_string_pretty(&generated_json).unwrap());

}