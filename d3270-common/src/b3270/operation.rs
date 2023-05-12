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

use serde::{Deserialize, Serialize};

// {"run":{"actions":[{"action":"Connect","args":["10.24.74.37:3270"]}]}}
// {"run":{"actions":[{"action":"Key","args":["a"]}]}}
// Operations
#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Run {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r_tag: Option<String>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    pub actions: Vec<Action>,
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Action {
    pub action: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Register {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help_params: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
/// Completes a passthru action unsuccessfully
#[serde(rename_all = "kebab-case")]
pub struct Fail {
    pub p_tag: String,
    pub text: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Succeed {
    pub p_tag: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub text: Vec<String>,
}
