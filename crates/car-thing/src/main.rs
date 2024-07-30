use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
}
