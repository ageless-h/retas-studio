use anyhow::Result;

pub fn print_help() {
    println!(r#"
RETAS Studio - Cross-platform 2D Animation Software

USAGE:
    retas [OPTIONS] [FILE]

OPTIONS:
    -h, --help       Print help information
    -v, --version    Print version information
    --info           Print system and GPU information

ARGS:
    <FILE>           Open the specified file (.cel, .dga, .scs)

EXAMPLES:
    retas                    Start with a new document
    retas animation.cel      Open an existing CEL file
    retas --info             Show system information

For more information, visit: https://github.com/user/retas-studio
"#);
}

pub fn print_info() -> Result<()> {
    println!("RETAS Studio v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("System Information:");
    println!("  OS: {}", std::env::consts::OS);
    println!("  Arch: {}", std::env::consts::ARCH);
    println!();
    println!("GPU Information:");
    println!("  (Run the application to see GPU details)");
    println!();

    Ok(())
}
