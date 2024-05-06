// Binary that generates the

mod add;
mod init;

use clap::{Parser, Subcommand};

use add::{do_add, AddCommand};
use init::{do_init, InitCommand};

#[derive(Debug, Parser)]
#[command(about, author = "Abhijit Gadgil", version)]
struct TestSetup;

#[derive(Debug, Parser)]
#[command(
    name = "cargo-xdp-tutorial",
    author = "Abhijit Gadgil",
    version,
    about,
    help_template(
        "\
{name}: v{version} by {author-with-newline}
{about-with-newline}
{usage}\n
Commands:\n{subcommands}"
    )
)]
enum Cli {
    #[clap(
        subcommand,
        help_template(
            "cargo-xdp-tutorial: v{version} by {author-with-newline}
{about-with-newline}
{usage}\n
Commands:\n{subcommands}"
        )
    )]
    XdpTutorial(Command),
}

#[derive(Debug, Subcommand)]
#[command(about, author = "Abhijit Gadgil", version)]
enum Command {
    #[clap(help_template(
        "cargo-xdp-tutorial: v{version} by {author-with-newline}
{about-with-newline}
{usage}\n
Options:\n{options}"
    ))]
    /// Initialize the project for tutorials.
    Init(InitCommand),

    #[clap(help_template(
        "cargo-xdp-tutorial: v{version} by {author-with-newline}
{about-with-newline}
{usage}\n
Options:\n{options}"
    ))]
    /// Setup test environment for running the tutorials.
    TestEnv(TestSetup),

    #[clap(help_template(
        "cargo-xdp-tutorial: v{version} by {author-with-newline}
{about-with-newline}
{usage}\n
Options:\n{options}"
    ))]
    /// Add individual tutorials to the project.
    Add(AddCommand),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::XdpTutorial(Command::Init(i)) => do_init(i),
        Cli::XdpTutorial(Command::Add(a)) => do_add(a),
        _ => todo!(),
    }
}
