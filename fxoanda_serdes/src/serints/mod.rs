use serde::{Deserialize, Deserializer, Serializer};

/// Serialize an Option<i32> as a string (if Some), or None.
pub fn serialize<S>(value: &Option<i32>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(ref v) = *value {
        serializer.collect_str(&v.to_string())
    } else {
        serializer.serialize_none()
    }
}

/// Deserialize an Option<i32> from a string or integer in JSON.
/// Accepts both string and integer representations.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        Int(i32),
        Str(String),
    }

    let opt = Option::<StringOrInt>::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(StringOrInt::Int(n)) => Ok(Some(n)),
        Some(StringOrInt::Str(s)) => s.parse::<i32>().map(Some).map_err(serde::de::Error::custom),
    }
}
