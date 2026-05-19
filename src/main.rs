use anstream::println;
use anyhow::Context;
use clap::Parser;
use imessage_database::{
    error::table::TableError,
    tables::{
        messages::Message,
        table::{Table, get_connection},
    },
    util::dirs::default_db_path,
};
use owo_colors::OwoColorize;
use regex::Regex;

/// Search your iMessages
#[derive(Parser)]
#[command(name = "bluegrep")]
struct Args {
    /// Pattern to search for (regex supported)
    pattern: String,
    #[arg(short = 'i', long)]
    ignore_case: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    search(&args.pattern, args.ignore_case)?;
    Ok(())
}

fn search(pattern: &str, ignore_case: bool) -> anyhow::Result<()> {
    let re = if ignore_case {
        Regex::new(&format!("(?i){pattern}"))?
    } else {
        Regex::new(pattern)?
    };

    let db = get_connection(&default_db_path())
        .context("Cannot open iMessage database — ensure full disk access is enabled for your terminal emulator in System Settings > Privacy & Security > Full Disk Access")?;

    Message::stream(&db, |msg| {
        if let Ok(mut msg) = msg {
            if let Ok(body) = msg.parse_body(&db) {
                msg.apply_body(body);
            }
            if let Some(text) = &msg.text {
                if let Some(m) = re.find(text) {
                    let (before, matched, after) = (
                        &text[..m.start()],
                        &text[m.start()..m.end()],
                        &text[m.end()..],
                    );
                    println!("{}{}{}", before, matched.red().bold(), after);
                }
            }
        }
        Ok::<(), TableError>(())
    })?;

    Ok(())
}
