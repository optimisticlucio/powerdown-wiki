use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer, Error as DeError};
use indexmap::IndexMap;

pub fn string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        Single(String),
        Multiple(Vec<String>),
    }

    match StringOrVec::deserialize(deserializer)? {
        StringOrVec::Single(s) => Ok(vec![s]),
        StringOrVec::Multiple(v) => Ok(v),
    }
}

pub fn deserialize_string_map<'de, D>(deserializer: D) -> Result<IndexMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrOther {
        String(String),
        Number(serde_json::Number),
        Bool(bool),
    }
    
    let map: IndexMap<String, StringOrOther> = IndexMap::deserialize(deserializer)?;
    
    Ok(map
        .into_iter()
        .map(|(k, v)| {
            let string_value = match v {
                StringOrOther::String(s) => s,
                StringOrOther::Number(n) => n.to_string(),
                StringOrOther::Bool(b) => b.to_string(),
            };
            (k, string_value)
        })
        .collect())
}