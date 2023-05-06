use serde::{Deserialize, Serialize};
use indication::{CodePage, ConnectAttempt, Connection, Erase, FileTransfer, Hello, Model, Oia, Passthru, Popup, Proxy, RunResult, Screen, ScreenMode, Scroll, Setting, Stats, TerminalName, Thumb, Tls, TlsHello, TraceFile, UiError};
use operation::{Fail, Register, Run, Succeed};

pub mod operation;
pub mod indication;
pub mod types;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Indication {
    Bell{}, // TODO: make sure this emits/parses {"bell": {}}
    /// Indicates that the host connection has changed state.
    Connection(Connection),
    /// A new host connection is being attempted
    ConnectAttempt(ConnectAttempt),
    /// Indicates the screen size
    Erase(Erase),
    /// Display switch between LTR and RTL
    Flipped {
        /// True if display is now in RTL mode
        value: bool,
    },
    /// An xterm escape sequence requested a new font
    Font {
        text: String,
    },
    /// The formatting state of the screen has changed.
    /// A formatted string has at least one field displayed.
    Formatted {
        state: bool,
    },
    /// File transfer state change
    #[serde(rename="ft")]
    FileTransfer(FileTransfer),
    /// An XTerm escape sequence requested a new icon name
    Icon{
        text: String,
    },
    /// The first message sent
    Initialize(Vec<InitializeIndication>),
    /// Change in the state of the Operator Information Area
    Oia(Oia),
    /// A passthru action has been invoked.
    /// Clients must respond with a succeed or fail operation
    Passthru(Passthru),
    /// Display an asynchronous message
    Popup(Popup),
    /// Result of a run operation
    RunResult(RunResult),
    /// Change to screen contents
    Screen(Screen),
    /// Screen dimensions/characteristics changed
    ScreenMode(ScreenMode),
    /// Screen was scrolled up by one row
    Scroll(Scroll),
    /// Setting changed
    Setting(Setting),
    /// I/O statistics
    Stats(Stats),
    /// Reports the terminal name sent to the host during TELNET negotiation
    TerminalName(TerminalName),
    /// Change in the scrollbar thumb
    Thumb(Thumb),
    /// Indicates the name of the trace file
    TraceFile(TraceFile),
    /// TLS state changed
    Tls(Tls),
    /// Error in b3270's input
    UiError(UiError),
    /// Xterm escape sequence requested a change to the window title
    WindowTitle{
        text: String,
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all="kebab-case")]
pub enum InitializeIndication {
    CodePages(Vec<CodePage>),
    /// Indicates that the host connection has changed state.
    Connection(Connection),
    /// Indicates the screen size
    Erase(Erase),
    /// The first element in the initialize array
    Hello(Hello),
    /// Indicates which 3270 models are supported
    Models(Vec<Model>),
    /// Change in the state of the Operator Information Area
    Oia(Oia),
    /// List of supported proxies
    Proxies(Vec<Proxy>),
    /// Set of supported prefixes
    Prefixes{value: String},
    /// Screen dimensions/characteristics changed
    ScreenMode(ScreenMode),
    /// Setting changed
    Setting(Setting),
    /// Indicates build-time TLS config
    TlsHello(TlsHello),
    /// TLS state changed
    Tls(Tls),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all="kebab-case")]
pub enum Operation {
    /// Run an action
    Run(Run),
    /// Register a pass-thru action
    Register(Register),
    /// Tell b3270 that a passthru action failed
    Fail(Fail),
    /// Tell b3270 that a passthru action succeeded
    Succeed(Succeed),
}

