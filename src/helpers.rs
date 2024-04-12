use std::collections::HashMap;
use serde_json::Value;

// helper functions to convert jsons to dictionaries
pub fn json_to_map(json: String) -> HashMap<String, Value> {
    let mut map = HashMap::new();
     if let Ok(parsed_json) = serde_json::from_str::<Value>(&json) {
         if let Some(obj) = parsed_json.as_object() {
             for (key, value) in obj {
                 map.insert(key.clone(), value.clone());
             }
         }
     }
     return map
}

pub fn json_vec_to_vec_map(json_vec: Vec<String>) -> Vec<HashMap<String, Value>> {
    let mut vec_map = Vec::new();
        
    for json_str in json_vec {
        let map = json_to_map(json_str);
        vec_map.push(map);
    }
    return vec_map
}





