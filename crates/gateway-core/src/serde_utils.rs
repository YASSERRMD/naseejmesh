use serde::de::{self, Visitor};
use std::fmt;

/// Custom deserializer to handle SurrealDB Record IDs
pub fn deserialize_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct IdVisitor;

    impl<'de> Visitor<'de> for IdVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or a SurrealDB record ID")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v.to_string())
        }

        fn visit_map<SimpleMap>(self, mut map: SimpleMap) -> Result<Self::Value, SimpleMap::Error>
        where
            SimpleMap: de::MapAccess<'de>,
        {
            let mut tb = None;
            let mut id = None;

            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "tb" => tb = Some(map.next_value::<String>()?),
                    "id" => {
                        // id can be various types in SurrealDB
                        let val: serde_json::Value = map.next_value()?;
                        id = Some(match val {
                            serde_json::Value::String(s) => s,
                            _ => val.to_string(),
                        });
                    }
                    _ => {
                        let _: de::IgnoredAny = map.next_value()?;
                    }
                }
            }

            match (tb, id) {
                (Some(t), Some(i)) => Ok(format!("{}:{}", t, i)),
                _ => Err(de::Error::custom("missing tb or id fields in record id")),
            }
        }
    }

    deserializer.deserialize_any(IdVisitor)
}

