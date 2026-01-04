use std::{fmt, str::FromStr};

use serde::{
    Deserializer, Serializer,
    de::{self, Visitor},
};

use chippie_emulator::ScreenResolution;

/// A special visitor, required by serde's Deserializer. It is needed to access deserializer's
/// internals in a safe way.
struct ResolutionVisitor;

impl<'de> Visitor<'de> for ResolutionVisitor {
    type Value = ScreenResolution;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string, which defines a resolution (NxN)")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if let Ok(resolution) = ScreenResolution::from_str(value) {
            return Ok(resolution);
        }

        Err(E::custom(format!(
            "The following value is not a resolution: {}",
            value
        )))
    }
}

/// Serialize an instance of the ScreenResolution enum using 'serde' by converting it to a string
pub fn serialize_resolution<S>(
    resolution: &ScreenResolution,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&resolution.to_string())
}

/// Derialize an instance of the ScreenResolution enum from string using 'serde'
pub fn deserialize_resolution<'a, D>(deserializer: D) -> Result<ScreenResolution, D::Error>
where
    D: Deserializer<'a>,
{
    deserializer.deserialize_str(ResolutionVisitor)
}
