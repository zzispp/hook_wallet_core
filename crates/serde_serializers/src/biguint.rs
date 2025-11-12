use num_bigint::BigUint;
use num_traits::Num;

use serde::{Deserialize, de};

pub fn serialize_biguint<S>(value: &BigUint, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn serialize_biguint_to_hex_str<S>(value: &BigUint, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&format!("0x{}", value.to_str_radix(16)))
}

pub fn deserialize_biguint_from_str<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;
    s.parse::<BigUint>().map_err(de::Error::custom)
}

pub fn deserialize_option_biguint_from_str<'de, D>(deserializer: D) -> Result<Option<BigUint>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(str_val) => str_val.parse::<BigUint>().map(Some).map_err(de::Error::custom),
        None => Ok(None),
    }
}

pub fn deserialize_biguint_from_hex_str<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;
    BigUint::from_str_radix(&s[2..], 16).map_err(serde::de::Error::custom)
}

pub fn deserialize_biguint_from_option_hex_str<'de, D>(deserializer: D) -> Result<Option<BigUint>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => match BigUint::from_str_radix(&s[2..], 16) {
            Ok(biguint) => Ok(Some(biguint)),
            Err(e) => Err(serde::de::Error::custom(e)),
        },
        None => Ok(None),
    }
}

pub fn serialize_option_biguint<S>(value: &Option<BigUint>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(v) => serializer.serialize_some(&v.to_string()),
        None => serializer.serialize_none(),
    }
}

pub fn biguint_from_hex_str(hex_str: &str) -> Result<BigUint, Box<dyn std::error::Error + Send + Sync>> {
    let hex_part = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    BigUint::from_str_radix(hex_part, 16).map_err(|e| format!("Invalid hex format: {}", e).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_biguint_from_hex_str() {
        assert_eq!(biguint_from_hex_str("0x1a").unwrap(), BigUint::from(26u32));
        assert_eq!(biguint_from_hex_str("1a").unwrap(), BigUint::from(26u32));
        assert_eq!(biguint_from_hex_str("0x0").unwrap(), BigUint::from(0u32));
        assert_eq!(biguint_from_hex_str("ff").unwrap(), BigUint::from(255u32));
        assert!(biguint_from_hex_str("xyz").is_err());
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestStruct {
        #[serde(serialize_with = "serialize_biguint", deserialize_with = "deserialize_biguint_from_str")]
        value: BigUint,
        #[serde(serialize_with = "serialize_option_biguint", deserialize_with = "deserialize_option_biguint_from_str")]
        optional_value: Option<BigUint>,
    }

    #[test]
    fn test_serialize_biguint() {
        let test = TestStruct {
            value: BigUint::from(12345678901234567890u128),
            optional_value: Some(BigUint::from(98765u64)),
        };
        let serialized = serde_json::to_string(&test).unwrap();
        assert_eq!(serialized, r#"{"value":"12345678901234567890","optionalValue":"98765"}"#);
    }

    #[test]
    fn test_deserialize_biguint() {
        let json_data = r#"{"value":"12345678901234567890","optionalValue":"98765"}"#;
        let deserialized: TestStruct = serde_json::from_str(json_data).unwrap();
        assert_eq!(deserialized.value, BigUint::from(12345678901234567890u128));
        assert_eq!(deserialized.optional_value, Some(BigUint::from(98765u64)));
    }

    #[test]
    fn test_serialize_none_biguint() {
        let test = TestStruct {
            value: BigUint::from(100u64),
            optional_value: None,
        };
        let serialized = serde_json::to_string(&test).unwrap();
        assert_eq!(serialized, r#"{"value":"100","optionalValue":null}"#);
    }

    #[test]
    fn test_deserialize_none_biguint() {
        let json_data = r#"{"value":"100","optionalValue":null}"#;
        let deserialized: TestStruct = serde_json::from_str(json_data).unwrap();
        assert_eq!(deserialized.value, BigUint::from(100u64));
        assert_eq!(deserialized.optional_value, None);
    }
}
