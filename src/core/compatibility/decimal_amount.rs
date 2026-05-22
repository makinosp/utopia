use std::str::FromStr;

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq)]
pub struct DecimalAmount(pub Decimal);

impl Serialize for DecimalAmount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format_amount(&self.0))
    }
}

impl<'de> Deserialize<'de> for DecimalAmount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let value = Decimal::from_str(&raw).map_err(serde::de::Error::custom)?;
        Ok(Self(value))
    }
}

fn format_amount(value: &Decimal) -> String {
    let normalized = value.normalize().to_string();

    match normalized.split_once('.') {
        Some((_, fractional)) if fractional.len() >= 2 => normalized,
        Some(_) => format!("{normalized}0"),
        None => format!("{normalized}.00"),
    }
}
