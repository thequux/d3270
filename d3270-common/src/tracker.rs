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

use std::collections::HashMap;
use tracing::warn;

use crate::b3270::indication::{Change, ComposeType, Connection, ConnectionState, CountOrText, Cursor, Erase, OiaField, OiaFieldName, Row, RunResult, Screen, ScreenMode, Scroll, Setting, Thumb, Tls, TraceFile};
use crate::b3270::types::{Color, GraphicRendition, PackedAttr};
use crate::b3270::{Indication, InitializeIndication};
use crate::b3270::types::Color::{NeutralBlack, NeutralWhite};

#[derive(Copy, Clone, Debug)]
pub struct CharCell {
    pub ch: char,
    pub attr: u32,
}
pub struct Tracker {
    screen: Vec<Vec<CharCell>>,
    oia: HashMap<OiaFieldName, OiaField>,
    screen_mode: ScreenMode,
    erase: Erase,
    thumb: Thumb,
    settings: HashMap<String, Setting>,

    // These are not init indications, but need to be tracked anyways
    cursor: Cursor,
    connection: Connection,
    formatted: bool,
    trace_file: Option<String>,
    tls: Option<Tls>,

    oia_tracker: OiaTracker,
    // These never change, but need to be represented in an initialize message
    static_init: Vec<InitializeIndication>,
}

#[derive(Clone, Debug)]
pub enum Disposition {
    // Deliver this indication to every connected client
    Broadcast,
    // Ignore this message
    Drop,
    // Send this message to one particular client
    Direct(String),
}

impl Tracker {
    pub fn handle_indication(&mut self, indication: &mut Indication) -> Disposition {
        match indication {
            Indication::Bell { .. }
            | Indication::ConnectAttempt(_)
            | Indication::Flipped { .. }
            | Indication::Font { .. }
            | Indication::Icon { .. }
            | Indication::Popup(_)
            | Indication::Stats(_)
            | Indication::WindowTitle { .. } => (),
            Indication::Connection(conn) => {
                self.connection = conn.clone();
            }
            Indication::Erase(erase) => {
                erase.fg = erase.fg.or(self.erase.fg).or(Some(NeutralWhite));
                erase.bg = erase.bg.or(self.erase.bg).or(Some(NeutralBlack));
                erase.logical_rows = erase.logical_rows.or(self.erase.logical_rows).or(Some(self.screen_mode.rows));
                erase.logical_cols = erase.logical_cols.or(self.erase.logical_cols).or(Some(self.screen_mode.columns));
                self.erase.logical_cols = erase.logical_cols;
                self.erase.logical_rows = erase.logical_rows;
                self.erase.fg = erase.fg;
                self.erase.bg = erase.bg;

                let rows = self.erase.logical_rows.unwrap() as usize;
                let cols = self.erase.logical_cols.unwrap() as usize;

                self.screen = vec![
                    vec![
                        CharCell {
                            attr: u32::c_pack(
                                erase.fg.unwrap_or(Color::NeutralBlack),
                                erase.bg.unwrap_or(Color::Blue),
                                GraphicRendition::empty(),
                            ),
                            ch: ' ',
                        };
                        cols
                    ];
                    rows
                ]
            }
            Indication::Formatted { state } => {
                self.formatted = *state;
            }

            Indication::Initialize(init) => {
                let mut static_init = Vec::with_capacity(init.len());
                for indicator in init.clone() {
                    match indicator {
                        InitializeIndication::CodePages(_)
                        | InitializeIndication::Hello(_)
                        | InitializeIndication::Models(_)
                        | InitializeIndication::Prefixes { .. }
                        | InitializeIndication::Proxies(_)
                        | InitializeIndication::TerminalName(_)
                        | InitializeIndication::TlsHello(_)
                        | InitializeIndication::Tls(_)
                        | InitializeIndication::TraceFile(_) => static_init.push(indicator),

                        // The rest are passed through to normal processing.
                        InitializeIndication::Thumb(thumb) => {
                            self.handle_indication(&mut Indication::Thumb(thumb));
                        }
                        InitializeIndication::Setting(setting) => {
                            self.handle_indication(&mut Indication::Setting(setting));
                        }
                        InitializeIndication::ScreenMode(mode) => {
                            self.handle_indication(&mut Indication::ScreenMode(mode));
                        }
                        InitializeIndication::Oia(oia) => {
                            self.handle_indication(&mut Indication::Oia(oia));
                        }
                        InitializeIndication::Erase(erase) => {
                            self.handle_indication(&mut Indication::Erase(erase));
                        }
                        InitializeIndication::Connection(conn) => {
                            self.handle_indication(&mut Indication::Connection(conn));
                        }
                    }
                }
            }
            Indication::Oia(oia) => {
                self.oia.insert(oia.field_name(), oia.clone());
                self.oia_tracker.notice(oia.clone());
            }
            Indication::Screen(screen) => {
                if let Some(cursor) = screen.cursor {
                    self.cursor = cursor;
                }
                for row in screen.rows.iter() {
                    let row_idx = row.row as usize - 1;
                    for change in row.changes.iter() {
                        let col_idx = change.column as usize - 1;
                        // update screen contents
                        let cols = self.screen[row_idx].iter_mut().skip(col_idx);
                        match change.change {
                            CountOrText::Count(n) => {
                                cols.take(n).for_each(|cell| {
                                    let mut attr = cell.attr;
                                    if let Some(fg) = change.fg {
                                        attr = attr.c_setfg(fg);
                                    }
                                    if let Some(bg) = change.bg {
                                        attr = attr.c_setbg(bg);
                                    }
                                    if let Some(gr) = change.gr {
                                        attr = attr.c_setgr(gr);
                                    }
                                    cell.attr = attr;
                                });
                            },
                            CountOrText::Text(ref text) => {
                                cols.zip(text.chars()).for_each(|(cell, ch)| {
                                    let mut attr = cell.attr;
                                    if let Some(fg) = change.fg {
                                        attr = attr.c_setfg(fg);
                                    }
                                    if let Some(bg) = change.bg {
                                        attr = attr.c_setbg(bg);
                                    }
                                    if let Some(gr) = change.gr {
                                        attr = attr.c_setgr(gr);
                                    }
                                    cell.attr = attr;
                                    cell.ch = ch;
                                });
                            }
                        }
                    }
                }
            }
            Indication::ScreenMode(mode) => {
                self.screen_mode = *mode;
                self.handle_indication(&mut Indication::Erase(Erase {
                    logical_rows: Some(self.screen_mode.rows),
                    logical_cols: Some(self.screen_mode.columns),
                    fg: None,
                    bg: None,
                }));
            }
            Indication::Scroll(Scroll { fg, bg }) => {
                let fg = fg.or(self.erase.fg).unwrap_or(Color::Blue);
                let bg = bg.or(self.erase.bg).unwrap_or(Color::NeutralBlack);
                let mut row = self.screen.remove(0);
                row.fill(CharCell {
                    attr: u32::c_pack(fg, bg, GraphicRendition::empty()),
                    ch: ' ',
                });
                self.screen.push(row);
            }
            Indication::Setting(setting) => {
                self.settings.insert(setting.name.clone(), setting.clone());
            }
            Indication::Thumb(thumb) => {
                self.thumb = thumb.clone();
            }
            Indication::TraceFile(TraceFile { name }) => {
                self.trace_file = name.clone();
            }
            Indication::Tls(tls) => {
                self.tls = Some(tls.clone());
            }

            // These need direction
            Indication::UiError(_) => {} // we can assume that this came from the last sent command
            Indication::Passthru(_) => {} // dunno how to handle this one
            Indication::FileTransfer(_) => {}
            Indication::RunResult(RunResult { r_tag, .. }) => {
                if let Some(dest) = r_tag {
                    return Disposition::Direct(dest.clone());
                } else {
                    return Disposition::Drop;
                }
            }
        }
        return Disposition::Broadcast;
    }

    pub fn get_init_indication(&self) -> Vec<Indication> {
        let mut contents = self.static_init.clone();
        contents.push(InitializeIndication::ScreenMode(self.screen_mode));
        contents.push(InitializeIndication::Erase(self.erase));
        contents.push(InitializeIndication::Thumb(self.thumb));

        contents.extend(self.oia.values().cloned().map(InitializeIndication::Oia));
        contents.extend(
            self.settings
                .values()
                .cloned()
                .map(InitializeIndication::Setting),
        );
        contents.extend(self.tls.clone().map(InitializeIndication::Tls));

        // Construct a screen snapshot
        let mut result = vec![
            Indication::Initialize(contents),
            Indication::Connection(self.connection.clone()),
            Indication::Screen(self.screen_snapshot()),
            Indication::Formatted {
                state: self.formatted,
            },
        ];
        if let Some(trace_file) = self.trace_file.clone() {
            result.push(Indication::TraceFile(TraceFile {
                name: Some(trace_file),
            }))
        }
        result
    }

    fn format_row(mut row: &[CharCell]) -> Vec<Change> {
        let mut result = vec![];
        let mut column = 1;
        while !row.is_empty() {
            let cur_gr = row[0].attr;

            let split_pt = row.iter().take_while(|ch| ch.attr == cur_gr).count();
            let (first, rest) = row.split_at(split_pt);
            row = rest;
            let content = first.iter().map(|cell| cell.ch).collect();
            result.push(Change {
                column,
                fg: Some(cur_gr.c_fg()),
                bg: Some(cur_gr.c_bg()),
                gr: Some(cur_gr.c_gr()),
                change: CountOrText::Text(content),
            });
            column += first.len() as u8;
        }

        result
    }

    fn screen_snapshot(&self) -> Screen {
        Screen {
            cursor: Some(self.cursor),
            rows: self
                .screen
                .iter()
                .map(Vec::as_slice)
                .map(Self::format_row)
                .enumerate()
                .map(|(row_id, changes)| Row {
                    row: row_id as u8 + 1,
                    changes,
                })
                .collect(),
        }
    }

    pub fn get_screen(&self) -> &Vec<Vec<CharCell>> {
        &self.screen
    }

    pub fn get_oia(&self) -> &HashMap<OiaFieldName, OiaField> {
        &self.oia
    }

    pub fn get_cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn get_oia_state(&self) -> &OiaTracker {
        &self.oia_tracker
    }

    pub fn get_connection(&self) -> &Connection { &self.connection }
}

#[derive(Default)]
pub struct OiaTracker {
    pub compose: Option<(ComposeType, String)>,
    pub insert: bool,
    pub lock: Option<String>,
    /// terminal, printer
    pub lu: Option<String>,
    pub not_undera: bool,
    pub printer_lu: Option<String>,
    pub reverse_input: bool,
    pub screen_trace: Option<usize>,
    pub script: bool,
    pub timing: Option<String>,
    pub typeahead: bool,
}

impl OiaTracker {
    pub fn notice(&mut self, oia: OiaField) {
        match oia {
            OiaField::Compose { value: true, type_: Some(typ), char:Some(ch_str) } => {
                self.compose = Some((typ, ch_str))
            }
            OiaField::Compose { value: false, ..} => self.compose = None,
            OiaField::Compose { .. } => warn!(?oia, "Unexpected OIA compose setting"),
            OiaField::Insert { value } => self.insert = value,
            OiaField::Lock { value } => self.lock = value,
            OiaField::Lu { value, lu } => {
                self.lu = Some(value);
                self.printer_lu = lu;
            }
            OiaField::NotUndera { value } => self.not_undera = value,
            OiaField::ReverseInput { value } => self.reverse_input = value,
            OiaField::ScreenTrace { value } => self.screen_trace = value,
            OiaField::Script { value } => self.script = value,
            OiaField::Timing { value } => self.timing = value,
            OiaField::Typeahead { value } => self.typeahead = value,
        };
    }

}

impl Default for Tracker {
    fn default() -> Self {
        let ret = Self {
            screen: vec![vec![CharCell{
                attr: u32::c_pack(Color::NeutralWhite, Color::NeutralBlack, GraphicRendition::empty()),
                ch: ' ',
            }]],
            oia: Default::default(),
            screen_mode: ScreenMode {
                columns: 80,
                rows: 43,
                color: true,
                model: 4,
                extended: true,
                oversize: false,
            },
            erase: Erase {
                logical_rows: None,
                logical_cols: None,
                fg: None,
                bg: None,
            },
            thumb: Thumb {
                top: 0.0,
                shown: 0.0,
                saved: 0,
                screen: 0,
                back: 0,
            },
            settings: Default::default(),
            cursor: Cursor {
                enabled: false,
                row: None,
                column: None,
            },
            connection: Connection {
                state: ConnectionState::NotConnected,
                host: None,
                cause: None,
            },
            formatted: false,
            trace_file: None,
            tls: None,
            static_init: vec![],
            oia_tracker: OiaTracker::default(),
        };
        ret
    }
}
