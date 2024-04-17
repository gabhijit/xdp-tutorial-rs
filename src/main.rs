// Binary that generates the

mod init;

use clap::Parser;

use init::{do_init, InitCommand};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Init(InitCommand),
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Init(i) => do_init(i),
    }
}
