mod app;
mod tudu;
use app::App;
use color_eyre::eyre::Result;
// use std::{env, process};

// const JSON_ARG: usize = 1;

fn main() -> Result<()> {
    // let args: Vec<String> = env::args().collect();

    // if args.len() != 2 {
    //     eprintln!("Usage: cargo run <json_file>");
    //     process::exit(1);
    // }
    // let json = load_json(&args[JSON_ARG]);

    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}
