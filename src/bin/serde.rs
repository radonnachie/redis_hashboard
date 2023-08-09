use std::{collections::{HashMap, HashSet}, vec, fmt};

use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap, Deserializer, de::{Visitor, MapAccess}};
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

#[derive(Debug, Hash, Serialize, Deserialize)]
#[serde(rename_all="lowercase")]
enum ClientActions {
    Drop,
    Request
}

#[derive(Debug)]
struct ClientAction {
    action: ClientActions,
    hash_names: HashSet<String>
}

impl Serialize for ClientAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        
        map.serialize_entry(
            &self.action,
            &self.hash_names
        )?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for ClientAction
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ClientActionVisitor;

        impl<'de> Visitor<'de> for ClientActionVisitor {
            type Value = ClientAction;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("single entry map<ClientAction, HashSet<String>>")
            }

            fn visit_map<V>(self, mut map: V) -> Result<ClientAction, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut action: Option<ClientActions> = None;
                let mut hash_names: Option<HashSet<String>> = None;
                while let Some(action_key) = map.next_key()? {
                    if action.is_some() {
                        return Err(serde::de::Error::duplicate_field("action"));
                    }
                    action = Some(action_key);
                    hash_names = Some(map.next_value()?);
                }
                let action = action.ok_or_else(|| serde::de::Error::missing_field("action"))?;
                let hash_names = hash_names.unwrap_or(HashSet::new());
                Ok(ClientAction {
                    action: action,
                    hash_names: hash_names
                })
            }
        }

        deserializer.deserialize_map(ClientActionVisitor)
    }
}

fn main() {
    let ca = ClientAction {
        action: ClientActions::Drop,
        hash_names: HashSet::from_iter(vec!["one", "two"].iter().map(|s| -> String {String::from(*s)}))
    };

    println!("{}", serde_json::to_string(&ca).unwrap());

    let ca_: ClientAction = serde_json::from_str(
        &serde_json::to_string(&ca).unwrap()
    ).unwrap();
    
    println!("{:?}", ca_);
    
    let ca_: ClientAction = serde_json::from_str(
        r#"
        {"drp": ["one"]}
        "#
    ).unwrap();
    println!("{:?}", ca_);
    return;

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