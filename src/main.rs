use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag="type")]
enum Model {
    Ref {
        model: String
    },
    Object {
        properties: Option<std::collections::BTreeMap<String, Model>>,
        value: Option<serde_yaml::Mapping>
    },
    Array {
        items: Option<Vec<Model>>,
        value: Option<serde_yaml::Sequence>
    },
    String {
        value: Option<String>
    },
    Integer {
        value: Option<i64>,
        min: Option<i64>,
        max: Option<i64>,
    },
    Float {
        value: Option<f64>,
        min: Option<f64>,
        max: Option<f64>,
    },
    Timestamp {
        format: Option<String>
    }
}

fn main() {
    let mut file = fs::File::open("config.yaml").expect("Failed to open input file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read input file");

    let config: serde_yaml::Value = serde_yaml::from_str(&contents).unwrap();    
    let data = &config["data"];
    let schemas = &config["components"]["schemas"];
    let constants = &config["components"]["constants"];
    let sequences = &config["components"]["sequences"];

    println!("{:?}", data);
    println!();
    let d : Model = serde_yaml::from_value(data.clone()).unwrap();
    println!("{:?}", d);
    
    println!();
    println!();
    println!("{}", serde_yaml::to_string(&d).unwrap());


    println!();
    println!();
    println!("{}", serde_json::to_string_pretty(&d).unwrap());

    //println!("{}", serde_json::to_string_pretty(schemas).unwrap());
}
