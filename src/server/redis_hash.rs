use std::{collections::{HashMap, HashSet}, fmt};

use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap, Deserializer, de::{Visitor, MapAccess}};

pub type RedisHashContents = HashMap<String, String>;

#[derive(Serialize)]
pub struct RedisHashContentsUpdate {
    upsert: RedisHashContents,
    delete: HashSet<String>
}

impl RedisHashContentsUpdate {
    pub fn from(
        contemporary: &RedisHashContents,
        previous: &RedisHashContents
    ) -> Option<RedisHashContentsUpdate> {
        let mut upsert = RedisHashContents::new();

        for (key, value) in contemporary.iter() {
            if let Some(previous_value) = previous.get(key) {
                if previous_value.eq(value) {
                    continue;
                }
            }

            upsert.insert(key.clone(), value.clone());
        }
        let delete: HashSet<String> = previous.keys()
            .filter(
                |k| !contemporary.contains_key(*k)
            ).map(
                |k| k.clone()
            ).collect();
        
        if delete.len() == 0 && upsert.len() == 0 {
            return None
        }
        Some(RedisHashContentsUpdate {
            upsert,
            delete
        })
    }
}


pub struct RedisHash {
    pub name: String,
    pub contents: RedisHashContents
}

impl Serialize for RedisHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        
        map.serialize_entry(
            &self.name,
            &self.contents
        )?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for RedisHash
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RedisHashVisitor;

        impl<'de> Visitor<'de> for RedisHashVisitor {
            type Value = RedisHash;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("single entry map<str, map<str, str>>")
            }

            fn visit_map<V>(self, mut map: V) -> Result<RedisHash, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name: Option<String> = None;
                let mut contents: Option<RedisHashContents> = None;
                while let Some(name_) = map.next_key()? {
                    if name.is_none() {
                        name = Some(name_);
                        contents = Some(map.next_value()?);
                    } else {
                        return Err(serde::de::Error::duplicate_field("name"));
                    }
                }
                let name = name.ok_or_else(|| serde::de::Error::missing_field("name"))?;
                let contents = contents.unwrap_or(RedisHashContents::new());
                Ok(RedisHash {
                    name: name,
                    contents: contents
                })
            }
        }

        deserializer.deserialize_map(RedisHashVisitor)
    }
}