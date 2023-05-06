use std::fmt::{Display, Formatter, Write};
use std::str::FromStr;
use bitflags::bitflags;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Visitor};

bitflags! {
    #[derive(Clone,Copy,Debug,PartialEq, Eq, Hash)]
    pub struct GraphicRendition: u16 {
        const UNDERLINE   = 0x001;
        const BLINK       = 0x002;
        const HIGHLIGHT   = 0x004;
        const SELECTABLE  = 0x008;
        const REVERSE     = 0x010;
        const WIDE        = 0x020;
        const ORDER       = 0x040;
        const PRIVATE_USE = 0x080;
        const NO_COPY     = 0x100;
        const WRAP        = 0x200;
    }
}

static FLAG_NAMES: &'static [(GraphicRendition, &'static str)] = &[
    (GraphicRendition::UNDERLINE, "underline"),
    (GraphicRendition::BLINK, "blink"),
    (GraphicRendition::HIGHLIGHT, "highlight"),
    (GraphicRendition::SELECTABLE, "selectable"),
    (GraphicRendition::REVERSE, "reverse"),
    (GraphicRendition::WIDE, "wide"),
    (GraphicRendition::ORDER, "order"),
    (GraphicRendition::PRIVATE_USE, "private-use"),
    (GraphicRendition::NO_COPY, "no-copy"),
    (GraphicRendition::WRAP, "wrap"),
];

impl Display for GraphicRendition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let flag_names = FLAG_NAMES.iter()
            .filter_map(|(val, name)| self.contains(*val).then_some(*name));
        for (n, name) in flag_names.enumerate() {
            if n != 0 {
                f.write_char(',')?;
            }
            f.write_str(name)?;
        }
        Ok(())
    }
}

impl Serialize for GraphicRendition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            serializer.serialize_u16(self.bits())
        }
    }
}

struct GrVisitor;


impl Visitor<'_> for GrVisitor {
    type Value = GraphicRendition;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "graphic rendition string or binary value")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> where E: Error {
        self.visit_u64((v & 0xFFFF) as u64)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> where E: Error {
        Ok(GraphicRendition::from_bits_truncate((v & 0xFFFF) as u16))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
        GraphicRendition::from_str(v).map_err(E::custom)
    }
}

impl FromStr for GraphicRendition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(",")
            .map(|attr| {
                FLAG_NAMES.iter()
                    .find(|(_, name)| *name == attr)
                    .map(|x| x.0)
                    .ok_or_else(|| format!("Invalid attr name {attr}"))
            })
            .collect()
    }
}

impl<'de> Deserialize<'de> for GraphicRendition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(GrVisitor)
        } else {
            deserializer.deserialize_u16(GrVisitor)
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use super::GraphicRendition;
    #[test]
    fn from_str_1() {
        assert_eq!(GraphicRendition::from_str("underline,blink"), Ok(GraphicRendition::BLINK | GraphicRendition::UNDERLINE))
    }
}