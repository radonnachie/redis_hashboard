use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase", tag = "shape")]
enum Shape {
    Circle {
        radius: f64,
    },
    Rectangle {
        length: f64,
        width: f64,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum Calculation {
    Perimeter,
    Area,
}

#[derive(Debug, Deserialize, Serialize)]
struct Request {
    calculation: Calculation,
    #[serde(flatten)]
    shape: Shape,
}

#[derive(Debug, Deserialize, Serialize)]
struct HashUpdate {
    #[serde(flatten)]
    hash:  HashMap<String, Value>
}

#[derive(Debug, Deserialize, Serialize)]
struct HashUpdateMap {
    #[serde(flatten)]
    hashes:  HashMap<String, HashUpdate>
}

#[derive(Deserialize)]
enum ClientActions {
    Drop,
    Request
}

#[derive(Deserialize)]
struct ClientAction {
    action: ClientActions,
    hash_names: HashSet<String>
}

fn main() {
    let json = r#"
        {
          "calculation": "perimeter",
          "shape": "rectangle",
          "length": 2.3,
          "width": 2.3
        }
    "#;

    let request: Request = serde_json::from_str(json).unwrap();

    println!("{:?}", request);
    println!("{:?}", request.shape);
    println!("{}", serde_json::to_string(&request).unwrap());


    let json = r#"
        {
            "hash1": {
                "key1": "1",
                "key2": 2
            },
            "hash2": {
                "key1": [1,2,3],
                "key2": "nil"
            }
        }
    "#;

    let hashupdate: HashUpdateMap = serde_json::from_str(json).unwrap();
    
    let val = json!("asdf");
    let hash = HashUpdate {
        hash: HashMap::from([
            (String::from("key1"), json!("1")),
            (String::from("key2"), json!(2)),
        ])
    };

    // let hashupdate_ = HashUpdateMap {
    //     hashes: HashMap::from([
    //         (
    //             String::from("hash1"),
    //             HashUpdate {
    //                 hash: HashMap::from([
    //                     ("key1", json!("1")),
    //                     ("key2", json!(2)),
    //                 ])
    //             }
    //         ),
    //         (
    //             String::from("hash2"),
    //             HashMap::from([
    //                 ("key1", json!("[1,2,3]")),
    //                 ("key2", json!("nil")),
    //             ])
    //         ),
    //     ])
    // }

    println!("{:?}", hashupdate);
    println!("{:?}", hashupdate.hashes);
    println!("{}", serde_json::to_string(&hashupdate).unwrap());
}