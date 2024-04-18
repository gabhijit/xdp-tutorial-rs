// Binary that generates the

mod init;

use clap::Parser;

use init::{do_init, InitCommand};

#[derive(Debug, Parser)]
struct TestSetup;

#[derive(Debug, Parser)]
struct AddTutorial;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    /// Initialize the project for tutorials.
    Init(InitCommand),

    /// Setup test environment for running the tutorials.
    TestEnv(TestSetup),

    /// Add individual tutorials to the project.
    Add(AddTutorial),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init(i) => do_init(i),
        _ => todo!(),
    }
}
