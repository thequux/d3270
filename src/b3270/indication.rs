use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
#[serde(rename_all="kebab-case")]
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
pub struct CodePage{
    /// The canonical name of the code page
    pub name: String,
    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub aliases: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Connection{
    /// New connection state
    pub state: ConnectionState,
    /// Host name, if connected
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub host: Option<String>,
    /// Source of the connection
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub cause: Option<ActionCause>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all="kebab-case")]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ComposeType {
    Std,
    Ge,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all="kebab-case")]
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
    Connected3270,
    ConnectedUnbound,
    ConnectedENvt,
    ConnectedESscp,
    #[serde(rename="connected-e-tn3270e")]
    ConnectedETn3270e,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all="kebab-case")]
pub struct Erase {
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub logical_rows: Option<u8>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub logical_cols: Option<u8>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub fg: Option<Color>,
    #[serde(default, skip_serializing_if="Option::is_none")]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
// This could be more typesafe, probably ¯\_(ツ)_/¯
pub struct Oia {
    #[serde(flatten)]
    pub field: OiaField,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub lu: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all="kebab-case", tag="field")]
#[derive(Debug, PartialEq, Clone)]
pub enum OiaField {
    /// Composite character in progress
    Compose{
        value: bool,
        #[serde(default, skip_serializing_if="Option::is_none")]
        char: Option<String>,
        #[serde(default, skip_serializing_if="Option::is_none")]
        type_: Option<ComposeType>,
    },
    /// Insert mode
    Insert{value: bool},
    /// Keyboard is locked
    Lock{
        #[serde(default, skip_serializing_if="Option::is_none")]
        value: Option<String>
    },
    Lu{
        /// Host session logical unit name
        value: String,
        /// Printer session LU name
        #[serde(default, skip_serializing_if="Option::is_none")]
        lu: Option<String>,
    },
    /// Communication pending
    NotUndera {
        value: bool,
    },
    PrinterSession {
        value: bool,
        /// Printer session LU name
        // TODO: determine if this is sent with this message or with Lu
        #[serde(default, skip_serializing_if="Option::is_none")]
        lu: Option<String>,
    },
    /// Reverse input mode
    ReverseInput {
        value: bool,
    },
    /// Screen trace count
    Screentrace {
        value: String,
    },
    /// Host command timer (minutes:seconds)
    Script {
        value: String,
    },
    Typeahead {
        value: bool,
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Proxy {
    pub name: String,
    pub username: String,
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Setting {
    pub name: String,
    /// I'd love something other than depending on serde_json for this.
    pub value: Option<serde_json::Value>,
    pub cause: ActionCause,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub struct ScreenMode {
    pub model: u8,
    pub rows: u8,
    pub cols: u8,
    pub color: bool,
    pub oversize: bool,
    pub extended: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TlsHello {
    pub supported: bool,
    pub provider: String, // docs claim this is always set, but I'm not sure.
    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub options: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all="kebab-case")]
pub struct Tls {
    pub secure: bool,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub verified: Option<bool>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub session: Option<String>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub host_cert: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all="kebab-case")]
pub struct ConnectAttempt {
    pub host_ip: String,
    pub port: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
// TODO: change this to an enum
pub struct Cursor {
    pub enabled: bool,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub row: Option<u8>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub column: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct FileTransfer {
    #[serde(flatten)]
    pub state: FileTransferState,
    pub cause: ActionCause,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all="lowercase", tag="state")]
pub enum FileTransferState {
    Awaiting,
    Running{
        /// Number of bytes transferred
        bytes: usize
    },
    Aborting,
    Complete{
        /// Completion message
        text: String,
        /// Transfer succeeded
        success: bool,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename="kebab-case")]
pub struct Passthru {
    pub p_tag: String,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub parent_r_tag: Option<String>,
    pub action: String,
    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Popup {
    #[serde(rename="type")]
    pub type_: PopupType,
    pub text: String,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub error: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
#[serde(rename="kebab-case")]
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
#[serde(rename="kebab-case")]
pub struct Row {
    pub row: u8,
    pub changes: Vec<Change>,
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename="kebab-case")]
pub enum CountOrText {
    Count(usize),
    Text(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Change {
    pub column: u8,
    #[serde(flatten)]
    pub change: CountOrText,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub fg: Option<Color>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub bg: Option<Color>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    /// Graphic rendition
    // TODO: parse comma-separated list of GR strings from https://x3270.miraheze.org/wiki/B3270/Graphic_rendition
    pub gr: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Screen {
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub cursor: Option<Cursor>,
    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub rows: Vec<Row>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename="kebab-case")]
pub struct RunResult {
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub r_tag: Option<String>,
    pub success: bool,
    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub text: Vec<String>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub abort: Option<bool>,
    /// Execution time in seconds
    pub time: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename="kebab-case")]
pub struct Scroll {
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub fg: Option<Color>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub bg: Option<Color>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
#[serde(rename="camelCase")]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename="kebab-case")]
pub struct Stats {
    pub bytes_received: usize,
    pub bytes_sent: usize,
    pub records_received: usize,
    pub records_sent: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TerminalName {
    pub text: String,
    #[serde(rename="override")]
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
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct UiError {
    pub fatal: bool,
    pub text: String,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub operation: Option<String>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub member: Option<String>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub line: Option<usize>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub column: Option<usize>,
}
