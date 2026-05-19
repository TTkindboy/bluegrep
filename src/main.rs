use anstream::println;
use clap::Parser;
use imessage_database::{
    error::table::TableError,
    tables::{
        messages::Message,
        table::{get_connection, Table},
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
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    search(&args.pattern)?;
    Ok(())
}

fn search(pattern: &str) -> anyhow::Result<()> {
    let re = Regex::new(pattern)?;
    let db = get_connection(&default_db_path())?;

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
