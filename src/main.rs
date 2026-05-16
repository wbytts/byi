use std::env;
use std::process::ExitCode;

use app::App;

mod app;
mod cli;
mod output;
mod sync;

fn main() -> ExitCode {
    match App::default().run(env::args().skip(1)) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
