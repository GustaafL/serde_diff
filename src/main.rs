use anyhow::Result;
use serde::de::value;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    metadata: Metadata,
    pub spec: Spec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    name: String,
    labels: HashMap<String, String>,
    expires: String,
    id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
    pub hostname: String,
}

#[derive(PartialEq)]
pub enum InnerType {
    InnerObject,
    InnerArray,
    InnerValue,
}

// first we establish the string is a valid json
// next we need an evaluate_object_or_array_or_value function that evaluates if the inner object is an object or an array or a value
// make a while loop that runs the evaluate_object_or_array_or_value function until the inner object is a value
// if it's an object we need to iterate through the keys and run the object_or_array_or_value function
// if it's an array we need to iterate through the array and evaluate if the value is an object, array or value

// a function that returns a serde json value
fn parse_json(json: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let parsed: Value = serde_json::from_str(json)?;
    // if the json is null return an error
    if parsed.is_null() {
        return Err("json is null".into());
    }
    Ok(parsed)
}

fn get_inner_object_type(value: &Value) -> InnerType {
    if value.is_object() {
        return InnerType::InnerObject;
    }
    if value.is_array() {
        return InnerType::InnerArray;
    }
    InnerType::InnerValue
}

fn compare_types(value: &Value, comparing_value: &Value) -> bool {
    // if value type is different from comparing_value type return
    if value.is_null() && comparing_value.is_null() {
        return true;
    }
    if value.is_number() && comparing_value.is_number() {
        return true;
    }
    if value.is_boolean() && comparing_value.is_boolean() {
        return true;
    }
    if value.is_string() && comparing_value.is_string() {
        return true;
    }
    if value.is_array() && comparing_value.is_array() {
        return true;
    }
    if value.is_object() && comparing_value.is_object() {
        return true;
    }
    false
}

fn hashmap_to_value(map: HashMap<String, Value>) -> Result<Value, Box<dyn Error>> {
    Ok(serde_json::to_value(map)?)
}

// compare values
fn compare_values(value: &Value, comparing_value: &Value) -> Result<Option<Value>, Box<dyn Error>> {
    if value == comparing_value {
        println!("Values are equal");
        Ok(None)
    } else {
        println!("Values are not equal");
        println!("value: {}", value);
        println!("Comparing value: {}", comparing_value);
        let mut different_values_map: HashMap<String, Value> = HashMap::new();
        different_values_map.insert("original_value".to_string(), value.clone());
        different_values_map.insert("comparing_value".to_string(), comparing_value.clone());
        let difference_value: Value = serde_json::to_value(different_values_map)?;
        Ok(Some(difference_value))
    }
}

// compare objects
fn compare_objects(value: &Value, comparing_value: &Value) -> Option<HashMap<String, Value>> {
    let mut new_map: HashMap<String, Value> = HashMap::new();
    let mut test_map: HashMap<String, Value> = HashMap::new();
    if let Some(inner_object) = value.as_object() {
        if let Some(comparing_object) = comparing_value.as_object() {
            for (key, inner_value) in inner_object {
                if comparing_object.get(key).is_none() {
                    new_map.insert(
                        "item_key_removed".to_string(),
                        serde_json::Value::String(key.to_string()),
                    );
                    let mut key_does_not_exist_map: HashMap<String, Value> = HashMap::new();
                    key_does_not_exist_map.insert(
                        "original_value_key_does_not_exist".to_string(),
                        value.clone(),
                    );
                    let key_does_not_exist_value = hashmap_to_value(key_does_not_exist_map).ok()?;

                    test_map.insert(key.to_string(), key_does_not_exist_value);
                    println!("Key: {} does not exist in comparing object", key);
                }
                if let Some(comparing_value) = comparing_object.get(key) {
                    if compare_types(inner_value, comparing_value) {
                        if get_inner_object_type(inner_value) != InnerType::InnerValue {
                            if let Some(evaluated_value) = evaluate(inner_value, comparing_value) {
                                test_map.insert(key.to_string(), evaluated_value);
                            }
                        } else {
                            if let Ok(Some(difference_map)) =
                                compare_values(inner_value, comparing_value)
                            {
                                test_map.insert(key.to_string(), difference_map);
                            }
                        }
                    } else {
                        println!("Types are not equal for key: {}", key);
                        println!("value: {}", inner_value);
                        println!("comparing_value{}", comparing_value);
                        let mut different_types_map: HashMap<String, Value> = HashMap::new();
                        different_types_map.insert("original_value".to_string(), value.clone());
                        different_types_map.insert(
                            "comparing_value_with_different_type".to_string(),
                            comparing_value.clone(),
                        );
                        let different_types_value = hashmap_to_value(different_types_map).ok()?; // test map looks as follows:
                        test_map.insert(key.to_string(), different_types_value);
                        // {
                        //    "key": {
                        //    "original_value": "value",
                        //    "comparing_value_with_different_type": "value"
                        //  }
                        //  "key2": {
                        //  "original_value_key_does_not_exist": "value",
                        //  }
                        //  "key3": {
                        //  "original_value": "value",
                        //  "comparing_value": "value"
                        //  }}
                    }
                }
            }
        }
    }
    Some(test_map)
}

// compare arrays
fn compare_arrays(value: &Value, comparing_value: &Value) -> Option<HashMap<String, Value>> {
    let mut test_map: HashMap<String, Value> = HashMap::new();
    let mut new_map: HashMap<String, Value> = HashMap::new();
    if let Some(inner_array) = value.as_array() {
        for (index, inner_value) in inner_array.iter().enumerate() {
            if let Some(comparing_array) = comparing_value.as_array() {
                if index < comparing_array.len() {
                    if compare_types(inner_value, &comparing_array[index]) {
                        if get_inner_object_type(inner_value) != InnerType::InnerValue {
                            if let Some(evaluated_array) =
                                evaluate(inner_value, &comparing_array[index])
                            {
                                test_map.insert(index.to_string(), evaluated_array);
                            }
                        } else {
                            if let Ok(Some(difference_map)) =
                                compare_values(inner_value, &comparing_array[index])
                            {
                                test_map.insert(index.to_string(), difference_map);
                            }
                            // return a vec with the index and value
                        }
                    } else {
                        if let Ok(Some(difference_map)) =
                            compare_values(inner_value, &comparing_array[index])
                        {
                            test_map.insert(index.to_string(), difference_map);
                        }

                        println!("Types are not equal for index: {}", index);
                        println!("Value: {}", inner_value);
                        println!("Comparing value: {}", comparing_array[index]);
                    }
                }
            }
        }
    }
    let array_values = hashmap_to_value(test_map).ok();
    if let Some(array_values) = array_values {
        new_map.insert("array_values".to_string(), array_values);
        Some(new_map)
    } else {
        None
    }
}

fn evaluate(object: &Value, comparing_object: &Value) -> Option<Value> {
    match get_inner_object_type(object) {
        InnerType::InnerObject => {
            if let Some(objects_map) = compare_objects(object, comparing_object) {
                if let Some(object_values) = hashmap_to_value(objects_map).ok() {
                    return Some(object_values);
                }
            }
        }

        InnerType::InnerArray => {
            if let Some(array_map) = compare_arrays(object, comparing_object) {
                if let Some(array_values) = hashmap_to_value(array_map).ok() {
                    return Some(array_values);
                }
            }
        }

        InnerType::InnerValue => {
            if let Some(Some(values_map)) = compare_values(object, comparing_object).ok() {
                return Some(values_map);
            }
        }
    }
    None
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Start recursive function");
    let json = r#"
    {
        "vector": [{
            "key": "value"
        }, {
            "key2": "value2"
        }, {
            "key3": ["value3", "value4"]
        }],
        "object": {
            "key3": "value3"
        },
        "bool": true,
        "number": 4,
        "string": "value5",
        "null_key": null,
        "metadata": {
            "name": "161bee39-cf07-4e31-90ba-6593c9f505cb",
            "labels": {
                "application": "api",
                "owner": "team_x"
            },
            "expires": "2021-12-06T20:49:04.136656523Z",
            "id": 1638823144137190452
        },
        "spec": {
            "hostname": "host1.example.com"
        }
    }
    "#;
    let json_compare = r#"
    {"bool":"default",
    "metadata": {
        "labels": {
            "application": "api",
            "owner": "team_"
            }
        }   
    }
    "#;

    let parsed: Value = parse_json(json)?;
    let parsed_compare: Value = parse_json(json_compare)?;
    if let Some(evaluated) = evaluate(&parsed, &parsed_compare) {
        println!("Evaluated: {}", evaluated);
    }
    Ok(())
}
