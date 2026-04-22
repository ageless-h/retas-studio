use anyhow::Result;

mod cli;
mod editor;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                cli::print_help();
            }
            "--version" | "-v" => {
                println!("RETAS Studio v{}", env!("CARGO_PKG_VERSION"));
            }
            "--info" => {
                cli::print_info()?;
            }
            path => {
                if path.starts_with('-') {
                    eprintln!("Unknown option: {}", path);
                    cli::print_help();
                    std::process::exit(1);
                }
                editor::run_with_file(path)?;
            }
        }
    } else {
        editor::run()?;
    }

    Ok(())
}
