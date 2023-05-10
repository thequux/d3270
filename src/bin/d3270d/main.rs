use std::ffi::OsString;
use std::str::FromStr;
use anyhow::{anyhow, bail};
use structopt::StructOpt;
use d3270::tracker::Tracker;

#[tokio::main]
fn main() -> anyhow::Result<()> {
    let mut subprocess_args = vec![
        OsString::from_str("-json").unwrap(),
    ];
    let mut args_iter = std::env::args_os().peekable();
    let mut connect_str = None;
    while let Some(arg) = args_iter.next() {
        // we default to one of the ignored args
        match arg.to_str().unwrap_or("-json") {
            "-json" | "-xml" | "-indent" | "--" |
            "-scriptportonce" | "-nowrapperdoc" |
            "-socket" | "-v" | "--version" => {}
            "-scriptport" | "-httpd" => {
                args_iter.next();
            }
            "-connect" => {
                connect_str = args_iter.next()
                    .ok_or_else(anyhow!("Arg required for -connect"))
                    .and_then(|arg| arg.into_string().map_err(|_| anyhow!("Invalid connect string")))
                    .map(Some)?;
            }
            "-e" => {
                'skip: while let Some(arg) = args_iter.peek() {
                    if arg.to_str().unwrap_or("").starts_with("-") {
                        break 'skip;
                    }
                    args_iter.next();
                }
            }
            _ => subprocess_args.push(arg),
        }
    }

    let connect_str = connect_str.ok_or_else(||anyhow!("No connect string given"))?;


    Ok(())
}

pub struct Subproc {
    tracker: Tracker,
}