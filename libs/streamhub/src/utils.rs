use serde::{Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, Copy)]
pub struct Uuid {
    value: uuid::Uuid,
}

impl Uuid {
    pub fn new() -> Self {
        Self {
            value: uuid::Uuid::new_v4(),
        }
    }

    pub fn from_str2(s: &str) -> Option<Self> {
        uuid::Uuid::from_str(s)
            .and_then(|u| Ok(Self { value: u }))
            .ok()
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {

    use super::Uuid;

    #[test]
    fn test_uuid() {
        let id = Uuid::new();

        let s = id.to_string();

        let serialized = serde_json::to_string(&id).unwrap();

        println!("serialized:{serialized}");

        println!("{s}");

        if let Some(u) = Uuid::from_str2(&s) {
            println!("{:?}", u.to_string());
        }
    }
}
