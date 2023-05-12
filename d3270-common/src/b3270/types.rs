/*************************************************************************
 * D3270 - Detachable 3270 interface                                      *
 * Copyright (C) 2023  Daniel Hirsch                                      *
 *                                                                        *
 * This program is free software: you can redistribute it and/or modify   *
 * it under the terms of the GNU General Public License as published by   *
 * the Free Software Foundation, either version 3 of the License, or      *
 * (at your option) any later version.                                    *
 *                                                                        *
 * This program is distributed in the hope that it will be useful,        *
 * but WITHOUT ANY WARRANTY; without even the implied warranty of         *
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the          *
 * GNU General Public License for more details.                           *
 *                                                                        *
 * You should have received a copy of the GNU General Public License      *
 * along with this program.  If not, see <https://www.gnu.org/licenses/>. *
 *************************************************************************/

use bitflags::bitflags;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter, Write};
use std::str::FromStr;

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
        if *self == Self::empty() {
           return f.write_str("default");
        }
        let flag_names = FLAG_NAMES
            .iter()
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_u64((v & 0xFFFF) as u64)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(GraphicRendition::from_bits_truncate((v & 0xFFFF) as u16))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        GraphicRendition::from_str(v).map_err(E::custom)
    }
}

impl FromStr for GraphicRendition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "default" {
            return Ok(GraphicRendition::empty());
        }
        s.split(",")
            .map(|attr| {
                FLAG_NAMES
                    .iter()
                    .find(|(_, name)| *name == attr)
                    .map(|x| x.0)
                    .ok_or_else(|| format!("Invalid GR attr name {attr}"))
            })
            .collect()
    }
}

impl<'de> Deserialize<'de> for GraphicRendition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(GrVisitor)
        } else {
            deserializer.deserialize_u16(GrVisitor)
        }
    }
}

#[cfg(test)]
mod test {
    use super::GraphicRendition;
    use std::str::FromStr;
    #[test]
    fn from_str_1() {
        assert_eq!(
            GraphicRendition::from_str("underline,blink"),
            Ok(GraphicRendition::BLINK | GraphicRendition::UNDERLINE)
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
#[serde(rename_all = "camelCase")]
#[repr(u8)]
pub enum Color {
    NeutralBlack,
    Blue,
    Red,
    Pink,
    Green,
    Turquoise,
    Yellow,
    NeutralWhite,
    Black,
    DeepBlue,
    Orange,
    Purple,
    PaleGreen,
    PaleTurquoise,
    Gray,
    White,
}

impl From<Color> for u8 {
    fn from(value: Color) -> Self {
        use Color::*;
        match value {
            NeutralBlack => 0,
            Blue => 1,
            Red => 2,
            Pink => 3,
            Green => 4,
            Turquoise => 5,
            Yellow => 6,
            NeutralWhite => 7,
            Black => 8,
            DeepBlue => 9,
            Orange => 10,
            Purple => 11,
            PaleGreen => 12,
            PaleTurquoise => 13,
            Gray => 14,
            White => 15,
        }
    }
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        use Color::*;
        match value & 0xF {
            0 => NeutralBlack,
            1 => Blue,
            2 => Red,
            3 => Pink,
            4 => Green,
            5 => Turquoise,
            6 => Yellow,
            7 => NeutralWhite,
            8 => Black,
            9 => DeepBlue,
            10 => Orange,
            11 => Purple,
            12 => PaleGreen,
            13 => PaleTurquoise,
            14 => Gray,
            15 => White,
            _ => unreachable!(),
        }
    }
}

pub trait PackedAttr {
    fn c_gr(self) -> GraphicRendition;
    fn c_fg(self) -> Color;
    fn c_bg(self) -> Color;
    fn c_setgr(self, gr: GraphicRendition) -> Self;
    fn c_setfg(self, fg: Color) -> Self;
    fn c_setbg(self, bg: Color) -> Self;
    fn c_pack(fg: Color, bg: Color, gr: GraphicRendition) -> Self;
}

impl PackedAttr for u32 {
    fn c_gr(self) -> GraphicRendition {
        GraphicRendition::from_bits_truncate((self & 0xFFFF) as u16)
    }

    fn c_fg(self) -> Color {
        ((self >> 16 & 0xF) as u8).into()
    }

    fn c_bg(self) -> Color {
        ((self >> 20 & 0xF) as u8).into()
    }

    fn c_setgr(self, gr: GraphicRendition) -> Self {
        self & !0xFFFF | gr.bits() as u32
    }

    fn c_setfg(self, fg: Color) -> Self {
        self & !0xF0000 | (u8::from(fg) as u32) << 16
    }

    fn c_setbg(self, bg: Color) -> Self {
        self & !0xF00000 | (u8::from(bg) as u32) << 20
    }

    fn c_pack(fg: Color, bg: Color, gr: GraphicRendition) -> Self {
        0.c_setfg(fg).c_setbg(bg).c_setgr(gr)
    }
}
