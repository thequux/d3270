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

use indication::{
    CodePage, ConnectAttempt, Connection, Erase, FileTransfer, Hello, Model, Passthru, Popup,
    Proxy, RunResult, Screen, ScreenMode, Scroll, Setting, Stats, TerminalName, Thumb, Tls,
    TlsHello, TraceFile, UiError,
};
use operation::{Fail, Register, Run, Succeed};
use serde::{Deserialize, Serialize};
use crate::b3270::indication::OiaField;

pub mod indication;
pub mod operation;
pub mod types;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all="kebab-case")]
pub enum Indication {
    Bell {}, // TODO: make sure this emits/parses {"bell": {}}
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
    #[serde(rename = "ft")]
    FileTransfer(FileTransfer),
    /// An XTerm escape sequence requested a new icon name
    Icon {
        text: String,
    },
    /// The first message sent
    Initialize(Vec<InitializeIndication>),
    /// Change in the state of the Operator Information Area
    Oia(OiaField),
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
    /// Change in the scrollbar thumb
    Thumb(Thumb),
    /// Indicates the name of the trace file
    TraceFile(TraceFile),
    /// TLS state changed
    Tls(Tls),
    /// Error in b3270's input
    UiError(UiError),
    /// Xterm escape sequence requested a change to the window title
    WindowTitle {
        text: String,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
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
    Oia(OiaField),
    /// Set of supported prefixes
    Prefixes {
        value: String,
    },
    /// List of supported proxies
    Proxies(Vec<Proxy>),
    /// Screen dimensions/characteristics changed
    ScreenMode(ScreenMode),
    /// Setting changed
    Setting(Setting),
    /// Reports the terminal name sent to the host during TELNET negotiation
    TerminalName(TerminalName),
    /// Scroll thumb position
    Thumb(Thumb),
    /// Indicates build-time TLS config
    TlsHello(TlsHello),
    /// TLS state changed
    Tls(Tls),
    /// Trace file
    TraceFile(TraceFile),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
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
