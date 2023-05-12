use std::fmt::Debug;
use std::io;
use std::io::Write;
use std::net::SocketAddr;
use std::ops::Range;
use std::os::linux::raw::stat;
use crossterm::{Command, cursor, queue, style, terminal};
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{Attribute};
use crossterm::terminal::ClearType;
use futures::StreamExt;
use tokio::net::TcpStream;
use structopt::StructOpt;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::select;
use d3270_common::b3270;
use d3270_common::b3270::{Indication, Operation};
use d3270_common::b3270::indication::{Connection, ConnectionState, Cursor, Erase, Screen};
use d3270_common::b3270::operation::{Action, Run};
use d3270_common::b3270::types::{Color, GraphicRendition, PackedAttr};
use d3270_common::b3270::types::Color::{NeutralBlack, NeutralWhite};
use d3270_common::tracker::Tracker;

macro_rules! actions {
    ($($aid:ident ( $($arg:expr),* $(,)?) ),+ $(,)?)=> {
        vec![
            $(
                Action{
                    action: stringify!($aid ).to_owned(),
                    args: vec![$($arg.to_string()),*]
                }
            ),+
        ]
    }
}


#[derive(StructOpt)]
struct Opts {
    host: SocketAddr,
}

#[derive(Default)]
pub struct State {
    tracker: Tracker,
    // cols, rows
    screen_size: (u16, u16),
}

trait IfElse {
    fn if_else<T>(&self, if_t: T, if_f: T) -> T;
}

impl IfElse for bool {
    fn if_else<T>(&self, if_t: T, if_f: T) -> T {
        if *self { if_t } else { if_f }
    }
}

fn debug_style<C: crossterm::Command+Debug>(styl: C) -> C {
    // eprintln!("styl: {styl:?}");
    // let mut rbuf = String::new();
    // styl.write_ansi(&mut rbuf).ok();
    // eprintln!("rendered: {:?}", rbuf);
    styl
}

impl State {
    fn apply_indicator(&mut self, mut ind: Indication) -> io::Result<()> {
        self.tracker.handle_indication(&mut ind);
        let mut buf = Vec::new();
        queue!(buf, crossterm::terminal::BeginSynchronizedUpdate)?;
        let empty_buf = buf.len();
        match ind {
            Indication::Oia(_) |
            Indication::Connection(_) |
            Indication::Initialize(_) => self.redraw_all()?,
            Indication::Screen(Screen{rows, ..}) => {
                for row in rows {
                    let row_n = row.row as usize - 1;
                    for upd in row.changes {
                        let col = upd.column as usize - 1;
                        let ncols = upd.change.len();
                        self.redraw_region(&mut buf, row_n, col..col+ncols)?;
                    }
                }
                self.redraw_oia(&mut buf)?;
                self.restore_cursor(&mut buf)?;
            }
            Indication::Erase(_) | Indication::ScreenMode(_) => {
                self.redraw_all()?; // this does its own writing
            }
            _ => {},
        }

        if buf.len() > empty_buf {
            queue!(buf, crossterm::terminal::EndSynchronizedUpdate)?;
            io::stdout().write_all(buf.as_slice())?;
            io::stdout().flush()?;
        }
        Ok(())
    }

    fn redraw_all(&self) -> io::Result<()> {
        let mut buf = Vec::new();
        queue!(buf,
            crossterm::terminal::BeginSynchronizedUpdate,
            style::SetBackgroundColor(style::Color::Rgb {r: 20, g: 20, b: 20}),
            terminal::Clear(ClearType::All),
        )?;
        let screen_len = self.tracker.get_screen().len();
        let screen_width = self.tracker.get_screen()[0].len();
        for i in 0..screen_len {
            self.redraw_region(&mut buf, i, 0..screen_width)?;
        }
        self.redraw_oia(&mut buf)?;

        queue!(buf, crossterm::terminal::EndSynchronizedUpdate)?;
        io::stdout().write_all(buf.as_slice())?;
        io::stdout().flush()?;
        Ok(())
    }

    fn redraw_oia(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        let screen_len = self.screen_size.1;

        let pos = match self.tracker.get_cursor() {
            Cursor { enabled: false, .. } => "  -/-  ".to_owned(),
            Cursor { enabled: true, row: Some(r), column: Some(c) } =>
                format!("{:3}/{:-3}", r-1, c-1),
            Cursor { enabled: true, row: None, column: Some(c) } =>
                format!("---/{:-3}", c-1),
            Cursor { enabled: true, row: Some(r), column: None } =>
                format!("{:3}/---", r-1),
            Cursor { enabled: true, row: None, column: None } => "---/---".to_owned(),
        };
        queue!(buf,
            cursor::MoveTo(1, screen_len-1),
            style::SetStyle(style::ContentStyle{
                foreground_color: Some(style::Color::Blue),
                background_color: Some(style::Color::Black),
                attributes: Attribute::OverLined.into(),
                underline_color: Some(style::Color::Cyan),
            }),
        )?;
        // print the OIA contents...
        let oia = self.tracker.get_oia_state();
        let status = if let Some(ref lock) = self.tracker.get_oia_state().lock {
            lock.as_str()
        } else {
            ""
        };
        let conn_ch = match self.tracker.get_connection().state {
            ConnectionState::NotConnected => " ",
            ConnectionState::Reconnecting => "~",
            ConnectionState::Resolving => "?",
            ConnectionState::TcpPending => ".",
            ConnectionState::TlsPending => "_",
            ConnectionState::TelnetPending => "t",
            ConnectionState::ConnectedNvt => "n",
            ConnectionState::ConnectedNvtCharmode => "C",
            ConnectionState::Connected3270 => "3",
            ConnectionState::ConnectedUnbound => "!",
            ConnectionState::ConnectedENvt => "N",
            ConnectionState::ConnectedSscp => "S",
            ConnectionState::ConnectedTn3270e => "E",
        };
        write!(
            buf,
            " {undera}{conn_ch}{status:-35} {compose:10}{ta}{rm}{im}{pr}{st}{sc} {lu:8} {timing:7} {pos}  ",
            undera = oia.not_undera.if_else('B', ' '),
            compose = match oia.compose {
                Some((ty, ref ch)) => format!("{ty:3?} {ch:6}"),
                None => "".to_owned(),
            },
            ta = oia.typeahead.if_else('T', ' '),
            rm = oia.reverse_input.if_else('R', ' '),
            im = oia.reverse_input.if_else('^', ' '),
            pr = oia.printer_lu.is_some().if_else('P', ' '),
            // security?
            st = oia.screen_trace.is_some().if_else('t', ' '),
            sc = oia.script.if_else('s', ' '),
            lu = oia.lu.as_ref().map(String::as_str).unwrap_or(""),
            timing = oia.timing.as_ref().map(String::as_str).unwrap_or(""),
        )?;
        queue!(buf,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine),
            crossterm::style::SetAttribute(Attribute::Reset)
        )?;
        self.restore_cursor(buf)
    }

    fn restore_cursor(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        match self.tracker.get_cursor() {
            Cursor { enabled: true, row: Some(row), column: Some(col)} => {
                queue!(buf,
                    crossterm::cursor::MoveTo(*col as u16, *row as u16),
                    crossterm::cursor::Show,
                )
            }
            _ => queue!(buf, crossterm::cursor::Hide)
        }
    }

    fn redraw_region(&self, buf: &mut Vec<u8>, row: usize, cols: Range<usize>) -> io::Result<()> {
        queue!(buf, cursor::MoveTo(cols.start as u16 + 1, row as u16 + 1)).ok();
        let mut last_attr = u32::c_pack(Color::NeutralWhite, Color::NeutralBlack, GraphicRendition::empty());
        for chr in &self.tracker.get_screen()[row][cols] {
            if chr.attr != last_attr {
                // eprintln!("chattr: {:08x}", chr.attr);
                queue!(buf,
                    style::SetAttribute(Attribute::Reset),
                    debug_style(style::SetForegroundColor(color_from_3270(chr.attr.c_fg()))),
                    // style::SetBackgroundColor(color_from_3270(chr.attr.c_bg())),
                )?;
                let c_gr = chr.attr.c_gr();
                for (gr, attr) in GR_TO_ATTR {
                    if c_gr.contains(*gr) {
                        queue!(buf, style::SetAttribute(*attr))?;
                    }
                }
                last_attr = chr.attr;
            }
            write!(buf, "{}", chr.ch)?;
        }
        queue!(buf, style::SetAttribute(Attribute::Reset))?;
        Ok(())
    }
}

static GR_TO_ATTR: &[(GraphicRendition, Attribute)] = &[
    (GraphicRendition::BLINK, Attribute::SlowBlink),
    (GraphicRendition::HIGHLIGHT, Attribute::Italic),
    (GraphicRendition::REVERSE, Attribute::Reverse),
    (GraphicRendition::ORDER, Attribute::Dim),
];

fn color_from_3270(color: b3270::types::Color) -> crossterm::style::Color {
    use crossterm::style::Color as CtColor;
    // TODO: make these RGB
    match color {
        Color::NeutralBlack => CtColor::Black,
        Color::Blue => CtColor::Blue,
        Color::Red => CtColor::Red,
        Color::Pink => CtColor::Magenta,
        Color::Green => CtColor::DarkGreen,
        Color::Turquoise => CtColor::Cyan,
        Color::Yellow => CtColor::Yellow,
        Color::NeutralWhite => CtColor::White,
        Color::Black => CtColor::Black,
        Color::DeepBlue => CtColor::DarkBlue,
        Color::Orange => CtColor::DarkYellow,
        Color::Purple => CtColor::DarkMagenta,
        Color::PaleGreen => CtColor::Green,
        Color::PaleTurquoise => CtColor::Cyan,
        Color::Gray => CtColor::Grey,
        Color::White => CtColor::White,
    }
}

mod term {
    use std::io;
    use std::io::Write;
    use crossterm::queue;
    pub struct TermSetup(bool);

    impl TermSetup {
        pub fn setup() -> io::Result<Self> {
            let mut stdout = io::stdout();
            queue!(stdout,
                crossterm::terminal::EnterAlternateScreen,
                crossterm::terminal::DisableLineWrap,
                crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            )?;
            stdout.flush()?;
            crossterm::terminal::enable_raw_mode()?;
            Ok(Self(false))
        }

        pub fn shutdown(&mut self) -> io::Result<()> {
            if self.0 { return Ok(()); }
            let mut stdout = io::stdout();
            crossterm::terminal::disable_raw_mode()?;
            queue!(stdout,
                crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
                crossterm::terminal::LeaveAlternateScreen,
                crossterm::terminal::EnableLineWrap,
            )?;
            stdout.flush()?;
            // avoid being called again in drop.
            self.0 = true;
            Ok(())
        }
    }

    impl Drop for TermSetup {
        fn drop(&mut self) {
            self.shutdown().ok();
        }
    }
}
use term::TermSetup;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut log_f = std::fs::File::create("b3270.trace.jsonl")?;
    let opts = Opts::from_args();
    let size = crossterm::terminal::size()?;
    let mut term_setup = TermSetup::setup()?;

    let mut remote = TcpStream::connect(opts.host).await?;
    let (rem_rd, mut rem_wr) = remote.split();
    let mut rem_rd = BufReader::new(rem_rd).lines();
    let mut state = State{
        tracker: Default::default(),
        screen_size: size,
    };

    state.apply_indicator(Indication::Connection(Connection{state: ConnectionState::NotConnected, host: None, cause: None}))?;

    let mut input = EventStream::new();

    'main: loop {
        select! {
            evt = rem_rd.next_line() => {
                let evt = if let Some(evt) = evt? { evt } else { break 'main; };
                log_f.write_all(evt.as_bytes())?;
                log_f.write_all(b"\n")?;
                let ind = match serde_json::from_str(evt.as_str()) {
                    Ok(ind) => ind,
                    Err(err) => {
                        term_setup.shutdown()?;
                        eprintln!("Error: {err:?}");
                        return Err(err.into());
                    }
                };
                state.apply_indicator(ind)?;
            },
            evt = input.next() => {
                let evt = if let Some(evt) = evt { evt } else { break 'main; };
                match evt? {
                    Event::Key(KeyEvent{code, modifiers, ..}) => {
                        let actions = match (code, modifiers) {
                            (KeyCode::Char('c'), KeyModifiers::CONTROL) => break 'main,
                            (KeyCode::Char(ch), KeyModifiers::NONE) => actions![Key(ch)],
                            (KeyCode::Char(ch), KeyModifiers::SHIFT) if ch.is_alphabetic() => actions![Key(ch.to_uppercase())],
                            (KeyCode::Backspace, _) => actions![BackSpace()],
                            (KeyCode::Enter, _) => actions![Enter()],
                            (KeyCode::F(n), KeyModifiers::NONE) => actions!(PF(n)),
                            (KeyCode::F(n), KeyModifiers::SHIFT) => actions!(PF(n+12)),
                            (KeyCode::Char('r'), KeyModifiers::CONTROL) => actions!(Reset()),
                            (KeyCode::Esc, KeyModifiers::NONE) => actions!(Attn()),
                            (KeyCode::Tab, KeyModifiers::NONE) => actions!(Tab()),
                            (KeyCode::Tab, KeyModifiers::SHIFT) |
                            (KeyCode::BackTab, _) => actions!(BackTab()),
                            (KeyCode::End, KeyModifiers::CONTROL) => actions!(EraseEOF()),
                            (KeyCode::Delete, KeyModifiers::NONE) => actions!(Delete()),
                            (KeyCode::Up, KeyModifiers::NONE) => actions!(Up()),
                            (KeyCode::Down, KeyModifiers::NONE) => actions!(Down()),
                            (KeyCode::Left, KeyModifiers::NONE) => actions!(Left()),
                            (KeyCode::Right, KeyModifiers::NONE) => actions!(Right()),
                            (KeyCode::PageUp, KeyModifiers::NONE) => actions!(Scroll("backward")),
                            (KeyCode::PageDown, KeyModifiers::NONE) => actions!(Scroll("forward")),
                            (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                                state.redraw_all()?;
                                continue 'main;
                            }
                            (_, _) => {
                                eprintln!("key {code:?} {modifiers:?}");
                                continue 'main;
                            },
                        };
                        let op = Operation::Run(Run{actions, type_: None, r_tag: None});
                        let mut enc = serde_json::to_string(&op)?;
                        enc.push('\n');
                        rem_wr.write_all(enc.as_bytes()).await?;
                    }

                    Event::Mouse(_) => {}
                    Event::Paste(_) => {}
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }
        }
    }
    term_setup.shutdown()?;
    Ok(())
}

