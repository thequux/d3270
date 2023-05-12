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

use crate::b3270::types::{Color, GraphicRendition};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum ActionCause {
    Command,
    Default,
    FileTransfer,
    Httpd,
    Idle,
    Keymap,
    Macro,
    None,
    Password,
    Paste,
    Peek,
    ScreenRedraw,
    Script,
    Typeahead,
    Ui,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CodePage {
    /// The canonical name of the code page
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Connection {
    /// New connection state
    pub state: ConnectionState,
    /// Host name, if connected
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    /// Source of the connection
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause: Option<ActionCause>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ComposeType {
    Std,
    Ge,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ConnectionState {
    NotConnected,
    Reconnecting,
    Resolving,
    TcpPending,
    TlsPending,
    TelnetPending,
    ConnectedNvt,
    ConnectedNvtCharmode,
    #[serde(rename="connected-3270")]
    Connected3270,
    ConnectedUnbound,
    ConnectedENvt,
    ConnectedSscp,
    ConnectedTn3270e,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub struct Erase {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logical_rows: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logical_cols: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fg: Option<Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg: Option<Color>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Hello {
    pub version: String,
    pub build: String,
    pub copyright: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Model {
    pub model: u8,
    pub rows: u8,
    pub columns: u8,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "field")]
#[derive(Debug, PartialEq, Clone)]
pub enum OiaField {
    /// Composite character in progress
    Compose {
        value: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        char: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        type_: Option<ComposeType>,
    },
    /// Insert mode
    Insert {
        value: bool,
    },
    /// Keyboard is locked
    Lock {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<String>,
    },
    Lu {
        /// Host session logical unit name
        value: String,
        /// Printer session LU name
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lu: Option<String>,
    },
    /// Communication pending
    NotUndera {
        value: bool,
    },
    // PrinterSession {
    //     value: bool,
    //     /// Printer session LU name
    //     // TODO: determine if this is sent with this message or with Lu
    //     #[serde(default, skip_serializing_if = "Option::is_none")]
    //     lu: Option<String>,
    // },
    /// Reverse input mode
    ReverseInput {
        value: bool,
    },
    /// Screen trace count
    ScreenTrace {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<usize>,
    },
    Script {
        value: bool,
    },
    /// Host command timer (minutes:seconds)
    Timing {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<String>,
    },
    Typeahead {
        value: bool,
    },
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub enum OiaFieldName {
    Compose,
    Insert,
    Lock,
    Lu,
    NotUndera,
    PrinterSession,
    ReverseInput,
    ScreenTrace,
    Script,
    Timing,
    Typeahead,
}

impl OiaField {
    pub fn field_name(&self) -> OiaFieldName {
        match self {
            OiaField::Compose { .. } => OiaFieldName::Compose,
            OiaField::Insert { .. } => OiaFieldName::Insert,
            OiaField::Lock { .. } => OiaFieldName::Lock,
            OiaField::Lu { .. } => OiaFieldName::Lu,
            OiaField::NotUndera { .. } => OiaFieldName::NotUndera,
            // OiaField::PrinterSession { .. } => OiaFieldName::PrinterSession,
            OiaField::ReverseInput { .. } => OiaFieldName::ReverseInput,
            OiaField::ScreenTrace { .. } => OiaFieldName::ScreenTrace,
            OiaField::Script { .. } => OiaFieldName::Script,
            OiaField::Timing { .. } => OiaFieldName::Timing,
            OiaField::Typeahead { .. } => OiaFieldName::Typeahead,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Proxy {
    pub name: String,
    pub username: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Setting {
    pub name: String,
    /// I'd love something other than depending on serde_json for this.
    pub value: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause: Option<ActionCause>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub struct ScreenMode {
    pub model: u8,
    pub rows: u8,
    pub columns: u8,
    pub color: bool,
    pub oversize: bool,
    pub extended: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TlsHello {
    pub supported: bool,
    pub provider: String, // docs claim this is always set, but I'm not sure.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Tls {
    pub secure: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_cert: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ConnectAttempt {
    pub host_ip: String,
    pub port: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
// TODO: change this to an enum
pub struct Cursor {
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct FileTransfer {
    #[serde(flatten)]
    pub state: FileTransferState,
    pub cause: ActionCause,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase", tag = "state")]
pub enum FileTransferState {
    Awaiting,
    Running {
        /// Number of bytes transferred
        bytes: usize,
    },
    Aborting,
    Complete {
        /// Completion message
        text: String,
        /// Transfer succeeded
        success: bool,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Passthru {
    pub p_tag: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_r_tag: Option<String>,
    pub action: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Popup {
    #[serde(rename = "type")]
    pub type_: PopupType,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum PopupType {
    /// Error message from a connection attempt
    ConnectError,
    /// Error message
    Error,
    /// Informational message
    Info,
    /// Stray action output (should not happen)
    Result,
    /// Output from the pr3287 process
    Printer,
    /// Output from other child process
    Child,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Row {
    pub row: u8,
    pub changes: Vec<Change>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum CountOrText {
    Count(usize),
    Text(String),
}

impl CountOrText {
    pub fn len(&self) -> usize {
        match self {
            CountOrText::Count(n) => *n,
            CountOrText::Text(text) => text.chars().count(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Change {
    pub column: u8,
    #[serde(flatten)]
    pub change: CountOrText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fg: Option<Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg: Option<Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Graphic rendition
    pub gr: Option<GraphicRendition>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Screen {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rows: Vec<Row>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct RunResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r_tag: Option<String>,
    pub success: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub text: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abort: Option<bool>,
    /// Execution time in seconds
    pub time: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Scroll {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fg: Option<Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg: Option<Color>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Stats {
    pub bytes_received: usize,
    pub bytes_sent: usize,
    pub records_received: usize,
    pub records_sent: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TerminalName {
    pub text: String,
    #[serde(rename = "override")]
    pub override_: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub struct Thumb {
    /// Fraction of scrollbar to top of thumb
    pub top: f32,
    /// Fraction of scrollbar for thumb display
    pub shown: f32,
    /// Number of rows saved
    pub saved: usize,
    /// Size of a screen in rows
    pub screen: usize,
    /// Number of rows scrolled back
    pub back: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TraceFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct UiError {
    pub fatal: bool,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn connection_state_serializes_as_expected() {
        assert_eq!(
            serde_json::to_string(&ConnectionState::ConnectedTn3270e).unwrap(),
            r#""connected-tn3270e""#
        );
        assert_eq!(
            serde_json::to_string(&ConnectionState::ConnectedSscp).unwrap(),
            r#""connected-sscp""#
        );
    }

    fn parse_row() {
        let instr = r#"[{"row":1,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","text":"z/OS V1R13 PUT Level 1401"},{"column":26,"fg":"red","gr":"highlight,selectable","count":26},{"column":52,"fg":"red","gr":"highlight,selectable","text":"IP Address = 10.24.74.32     "}]},{"row":2,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":51},{"column":52,"fg":"red","gr":"highlight,selectable","text":"VTAM Terminal = SC0TCP05     "}]},{"row":3,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":80}]},{"row":4,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":23},{"column":24,"fg":"red","gr":"highlight,selectable","text":"Application Developer System"},{"column":52,"fg":"red","gr":"highlight,selectable","count":29}]},{"row":5,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":80}]},{"row":6,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":32},{"column":33,"fg":"red","gr":"highlight,selectable","text":"//  OOOOOOO   SSSSS"},{"column":52,"fg":"red","gr":"highlight,selectable","count":29}]},{"row":7,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":31},{"column":32,"fg":"red","gr":"highlight,selectable","text":"//  OO    OO SS"},{"column":47,"fg":"red","gr":"highlight,selectable","count":34}]},{"row":8,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":23},{"column":24,"fg":"red","gr":"highlight,selectable","text":"zzzzzz //  OO    OO SS"},{"column":46,"fg":"red","gr":"highlight,selectable","count":35}]},{"row":9,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":25},{"column":26,"fg":"red","gr":"highlight,selectable","text":"zz  //  OO    OO SSSS"},{"column":47,"fg":"red","gr":"highlight,selectable","count":34}]},{"row":10,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":23},{"column":24,"fg":"red","gr":"highlight,selectable","text":"zz   //  OO    OO      SS"},{"column":49,"fg":"red","gr":"highlight,selectable","count":32}]},{"row":11,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":21},{"column":22,"fg":"red","gr":"highlight,selectable","text":"zz    //  OO    OO      SS"},{"column":48,"fg":"red","gr":"highlight,selectable","count":33}]},{"row":12,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":19},{"column":20,"fg":"red","gr":"highlight,selectable","text":"zzzzzz //   OOOOOOO  SSSS"},{"column":45,"fg":"red","gr":"highlight,selectable","count":36}]},{"row":13,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":80}]},{"row":14,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":80}]},{"row":15,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":19},{"column":20,"fg":"red","gr":"highlight,selectable","text":"System Customization - ADCD.Z113H.*"},{"column":55,"fg":"red","gr":"highlight,selectable","count":26}]},{"row":16,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":80}]},{"row":17,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":80}]},{"row":18,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":80}]},{"row":19,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":80}]},{"row":20,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","text":" ===> Enter \"LOGON\" followed by the TSO userid. Example \"LOGON IBMUSER\" or      "}]},{"row":21,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","text":" ===> Enter L followed by the APPLID"},{"column":37,"fg":"red","gr":"highlight,selectable","count":44}]},{"row":22,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","text":" ===> Examples: \"L TSO\", \"L CICSTS41\", \"L CICSTS42\", \"L IMS11\", \"L IMS12\"       "}]},{"row":23,"changes":[{"column":1,"fg":"red","gr":"highlight,selectable","count":79},{"column":80,"fg":"green","count":1}]},{"row":24,"changes":[{"column":1,"fg":"green","count":79},{"column":80,"fg":"red","gr":"highlight,selectable","count":1}]}]"#;
        if let Err(err) = serde_json::from_slice::<Vec<Row>>(instr.as_bytes()) {
            let pos = err.column();
            println!("Parse error: {err}");
            let (pre, post) = instr.split_at(err.column());
            println!("Context: {pre}\x1b[1;31m{post}\x1b[0m");
            panic!("{}", err);
        }
    }
}