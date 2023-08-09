use std::{collections::HashSet, fmt};

use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap, Deserializer, de::{Visitor, MapAccess}};

#[derive(Debug, Hash, Serialize, Deserialize)]
#[serde(rename_all="lowercase")]
pub enum ClientActions {
    Drop,
    Request
}

#[derive(Debug)]
pub struct ClientAction {
    pub action: ClientActions,
    pub hash_names: HashSet<String>
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