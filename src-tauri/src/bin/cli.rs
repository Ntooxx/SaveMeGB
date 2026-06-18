use savemegb_lib::{run_cli_refresh, run_cli_scan_with};
use std::process::ExitCode;

fn main() -> ExitCode {
    let _ = env_logger::try_init();
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("scan");
    match cmd {
        "scan" => {
            let mode = args.get(2).map(String::as_str).unwrap_or("standard");
            match run_cli_scan_with(mode) {
                Ok(report) => {
                    println!("{}", serde_json::to_string_pretty(&report).unwrap());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("scan failed: {e}");
                    ExitCode::from(2)
                }
            }
        }
        "refresh" => match run_cli_refresh() {
            Ok(_) => { eprintln!("manifest refreshed"); ExitCode::SUCCESS }
            Err(e) => { eprintln!("refresh failed: {e}"); ExitCode::from(3) }
        },
        "version" => { println!("savelock-cli 0.1.0"); ExitCode::SUCCESS }
        other => {
            eprintln!("unknown command: {other}");
            eprintln!("usage: savelock-cli <scan [quick|standard|deep]|refresh|version>");
            ExitCode::from(1)
        }
    }
}
