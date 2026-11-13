// Unter Windows im Release kein zusätzliches Konsolenfenster öffnen.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("Amber Player {}\n\n{}", env!("CARGO_PKG_VERSION"), amber_player_lib::HELP);
        return;
    }
    if args.iter().any(|a| a == "--headless") {
        if let Err(e) = amber_player_lib::run_headless(&args) {
            eprintln!("Fehler: {e:#}");
            std::process::exit(1);
        }
        return;
    }
    amber_player_lib::run();
}
