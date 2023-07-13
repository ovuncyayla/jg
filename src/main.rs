use rand::Rng;
use serde::{Deserialize, Serialize};
use std::borrow::BorrowMut;
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Model {
    Ref {
        path: String,
    },
    Object {
        properties: Option<std::collections::BTreeMap<String, Model>>,
        value: Option<serde_yaml::Mapping>,
    },
    Array {
        items: Option<Vec<Model>>,
        value: Option<serde_yaml::Sequence>,
    },
    String {
        value: Option<String>,
    },
    Integer {
        min: Option<i64>,
        max: Option<i64>,
        value: Option<i64>,
    },
    Float {
        min: Option<f64>,
        max: Option<f64>,
        value: Option<f64>,
    },
    Timestamp {
        format: Option<String>,
        value: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Sequence {
    StrSeq {
        #[serde(default)]
        prefix: String,
        #[serde(default)]
        suffix: String,
        #[serde(default)]
        start: i64,
    },
    IntSeq {
        #[serde(default)]
        start: i64,
    },
    UUID4Seq,
}

#[derive(Serialize, Deserialize, Debug)]
struct Components {
    #[serde(default)]
    models: BTreeMap<String, Model>,
    #[serde(default)]
    constants: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    sequences: BTreeMap<String, Sequence>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Meta {
    size: i32,
    prettify: bool,
}

impl Default for Meta {
    fn default() -> Self {
        Meta {
            size: 1,
            prettify: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    #[serde(default)]
    meta: Meta,
    schema: Model,
    components: Option<Components>,
}

fn generate_json(config: &Config) -> Result<serde_json::Value, String> {
    let schema = &config.schema;
    //let components = &config.components;

    let mut seq_ctx: BTreeMap<String, i64> = BTreeMap::new();
    generate_json_for_model(schema, config, seq_ctx.borrow_mut())
    //Ok(serde_json::Value::default())
}

fn generate_json_for_model(
    model: &Model,
    config: &Config,
    seq_ctx: &mut BTreeMap<String, i64>,
) -> Result<serde_json::Value, String> {
    match model {
        Model::Ref { path: ref_name } => {
            if let Some(components) = &config.components {
                // Find component type
                match ref_name.split("/").collect::<Vec<&str>>()[..] {
                    [_, ref_type, ref_key] => match ref_type {
                        "models" => {
                            let model = components.models.get(ref_key).expect(
                                format!(
                                    "Can not find model definition. Required by ref: {}",
                                    ref_name
                                )
                                .as_str(),
                            );
                            return generate_json_for_model(model, config, seq_ctx);
                        }
                        "constants" => {
                            let constant = components.constants.get(ref_key).expect(
                                format!(
                                    "Can not find constant definition. Required by ref: {}",
                                    ref_name
                                )
                                .as_str(),
                            );
                            let json_val = serde_json::to_value(constant).expect(
                                format!(
                                    "Error while converting yaml value of {} to json value",
                                    ref_key
                                )
                                .as_str(),
                            );
                            return Ok(json_val);
                        }
                        "sequences" => {
                            let mut sequence = components.sequences.get(ref_key).expect(
                                format!(
                                    "Can not find constant definition. Required by ref: {}",
                                    ref_name
                                )
                                .as_str(),
                            );

                            match &mut sequence {
                                Sequence::StrSeq {
                                    prefix,
                                    suffix,
                                    start,
                                } => {
                                    let val = seq_ctx.entry(ref_key.to_string()).or_insert(*start);
                                    *val = *val + 1;
                                    return Ok(serde_json::to_value(format!(
                                        "{}{}{}",
                                        *prefix, val, *suffix
                                    ))
                                    .expect(
                                        format!(
                                            "Error while converting seq. value to json {}",
                                            ref_key
                                        )
                                        .as_str(),
                                    ));
                                }
                                Sequence::IntSeq { start } => {
                                    let val = seq_ctx.entry(ref_key.to_string()).or_insert(*start);
                                    *val = *val + 1;
                                    return Ok(serde_json::to_value(val).expect(
                                        format!(
                                            "Error while converting seq. value to json {}",
                                            ref_key
                                        )
                                        .as_str(),
                                    ));
                                }
                                Sequence::UUID4Seq => {
                                    return Ok(serde_json::to_value(Uuid::new_v4().to_string())
                                        .expect(
                                            format!(
                                                "Error while converting seq. value to json {}",
                                                ref_key
                                            )
                                            .as_str(),
                                        ));
                                }
                            }
                        }
                        _ => return Err(format!("Invalid ref type: {}", ref_type)),
                    },
                    _ => return Err(format!("Error while parsing ref string: {}", ref_name)),
                }
            }

            Err(format!(
                "Can not find component definitions in config file. Required by ref: {}",
                ref_name
            ))
        }
        Model::Object { properties, value } => {
            if let Some(mapping) = value {
                let map_as_value: serde_yaml::Value =
                    serde_yaml::to_value(mapping).expect("Error while serializing object value");
                let map: serde_json::Value = serde_yaml::from_value(map_as_value)
                    .expect("Error while converting yaml to json value");
                return Ok(map);
            }

            if let Some(props) = properties {
                let mut json_obj = serde_json::Map::new();
                for (key, prop_schema) in props {
                    let value = generate_json_for_model(prop_schema, config, seq_ctx).expect(
                        format!("Error while generating object property: {}", key).as_str(),
                    );
                    json_obj.insert(key.clone(), value);
                }
                return Ok(serde_json::to_value(json_obj).expect(
                    "Generic serialization error occured while converting map to json value",
                ));
            }

            Err(
                "Either proprties or value field must be defined for model type 'Object'"
                    .to_string(),
            )
        }
        Model::Array { items, value } => {
            if let Some(items) = items {
                let mut json_array: Vec<serde_json::Value> = Vec::new();
                for item_schema in items {
                    let item_value = generate_json_for_model(item_schema, config, seq_ctx)
                        .expect("Error while generating array element");
                    json_array.push(item_value);
                }
                return Ok(serde_json::Value::Array(json_array));
            }

            if let Some(sequence) = value {
                let mut json_array: Vec<serde_json::Value> = Vec::new();
                for s in sequence {
                    let json_val: serde_json::Value = serde_yaml::from_value(s.clone())
                        .expect("Error while converting yaml value to json");
                    json_array.push(json_val);
                }
                return Ok(serde_json::Value::Array(json_array));
            }

            Err("Either items or value field must be defined for model type 'Array'".to_string())
        }
        Model::String { value } => {
            if let Some(value) = value {
                Ok(serde_json::Value::String(value.clone()))
            } else {
                Ok(serde_json::Value::Null)
            }
        }
        Model::Integer { value, min, max } => {
            if let Some(value) = value {
                Ok(serde_json::Value::Number(value.clone().into()))
            } else {
                let mut rng = rand::thread_rng();
                let res = match (min, max) {
                    (None, None) => rng.gen(),
                    (None, Some(max)) => rng.gen_range(i64::MIN..*max),
                    (Some(min), None) => rng.gen_range(*min..i64::MAX),
                    (Some(min), Some(max)) => rng.gen_range(*min..*max),
                };

                Ok(serde_json::to_value(res).expect("Error while generating random integer"))
            }
        }
        Model::Float { value, min, max } => {
            if let Some(value) = value {
                Ok(serde_json::Value::from(*value))
            } else {
                let mut rng = rand::thread_rng();
                let res = match (min, max) {
                    (None, None) => rng.gen(),
                    (None, Some(max)) => rng.gen_range(f64::MIN..*max),
                    (Some(min), None) => rng.gen_range(*min..f64::MAX),
                    (Some(min), Some(max)) => rng.gen_range(*min..*max),
                };

                Ok(serde_json::to_value(res).expect("Error while generating random float"))
            }
        }
        Model::Timestamp {
            value,
            format: _format,
        } => {
            // Handle timestamp generation logic here

            if let Some(value) = value {
                return Ok(serde_json::from_str(value)
                    .expect("Error while parsing value field for timestamp"));
            }

            Ok(serde_json::to_value(chrono::Utc::now().to_rfc3339())
                .expect("Generic error while generating timestamp"))
        }
    }
}

fn main() {
    let mut file = fs::File::open("config.yaml").expect("Failed to open input file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read input file");

    let mut config: Config = serde_yaml::from_str(&contents).unwrap();

    let qwe = generate_json(&mut config);

    // println!("{:#?}", config);
    // println!();
    // println!();
    // println!();
    // println!("{:#?}", qwe);
    println!("{}", serde_json::to_string_pretty(&qwe.unwrap()).unwrap().to_string());
}
