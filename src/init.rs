// Implementation of tutorial initialization

use clap::Parser;

#[derive(Debug, Parser)]
pub(crate) struct InitCommand {
    /// A Path to local directory or a git repository containing XDP Tutorial Templates.
    #[clap(name = "template-path", required = true)]
    template_path: std::path::PathBuf,

    /// A Path to local directory where the templates should be instantiated
    #[clap(name = "tutorial-path", default_value = ".")]
    tutorial_path: std::path::PathBuf,
}

pub(crate) fn do_init(init_command: InitCommand) {
    eprintln!("Init Command: {init_command:#?}");
}
