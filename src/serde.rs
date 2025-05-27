/*
	DrCr: Web-based double-entry bookkeeping framework
	Copyright (C) 2022-2025  Lee Yingtong Li (RunasSudo)

	This program is free software: you can redistribute it and/or modify
	it under the terms of the GNU Affero General Public License as published by
	the Free Software Foundation, either version 3 of the License, or
	(at your option) any later version.

	This program is distributed in the hope that it will be useful,
	but WITHOUT ANY WARRANTY; without even the implied warranty of
	MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
	GNU Affero General Public License for more details.

	You should have received a copy of the GNU Affero General Public License
	along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

/// Serialises [chrono::NaiveDateTime] in database format
///
/// Use as `#[serde(with = "crate::serde::naivedatetime_to_js")]`, etc.
pub mod naivedatetime_to_js {
	use std::fmt;

	use chrono::NaiveDateTime;
	use serde::{
		de::{self, Unexpected, Visitor},
		Deserializer, Serializer,
	};

	pub(crate) fn serialize<S: Serializer>(
		dt: &NaiveDateTime,
		serializer: S,
	) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&dt.format("%Y-%m-%d %H:%M:%S%.6f").to_string())
	}

	struct DateVisitor;
	impl<'de> Visitor<'de> for DateVisitor {
		type Value = NaiveDateTime;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			write!(formatter, "a date string")
		}

		fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
		where
			E: de::Error,
		{
			match NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.6f") {
				Ok(dt) => Ok(dt),
				Err(_) => Err(de::Error::invalid_value(Unexpected::Str(s), &self)),
			}
		}
	}

	pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
		deserializer: D,
	) -> Result<NaiveDateTime, D::Error> {
		deserializer.deserialize_str(DateVisitor)
	}
}
