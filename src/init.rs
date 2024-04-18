// Implementation of tutorial initialization

use clap::Parser;

#[derive(Debug, Parser)]
pub(crate) struct InitCommand {
    /// A Path to local directory or a git repository containing XDP Tutorial Templates.
    #[clap(name = "template-path")]
    template_path: std::path::PathBuf,

    /// Path inside the root of the repository, if the given template path is a GIT repository.
    #[clap(name = "repository-template-path", short = 'r', long = "path")]
    repository_template_path: Option<std::path::PathBuf>,

    /// Git Reference if the template-path is a repository (branch, tag or commit hash)
    #[clap(name = "git-ref", short = 't')]
    git_ref: Option<String>,

    /// A Path to local directory where the templates should be instantiated
    #[clap(name = "tutorial-path", short = 'o', default_value = ".")]
    tutorial_path: String,
}

pub(crate) fn do_init(init_command: InitCommand) -> anyhow::Result<()> {
    eprintln!("Init Command: {init_command:#?}");

    let scaffold_ops = cargo_scaffold::Opts::builder(init_command.template_path);

    let scaffold_ops = if let Some(repository_template_path) = init_command.repository_template_path
    {
        scaffold_ops.repository_template_path(repository_template_path)
    } else {
        scaffold_ops
    };

    let scaffold_ops = if let Some(git_ref) = init_command.git_ref {
        scaffold_ops.repository_template_path(git_ref)
    } else {
        scaffold_ops
    };

    let project_name = if init_command.tutorial_path.as_str() == "." {
        "xdp-tutorial-rust-sol".into()
    } else {
        init_command.tutorial_path
    };

    let scaffold_ops = scaffold_ops.project_name(project_name);

    eprintln!("scaffold_opts: {scaffold_ops:#?}");

    let scaffold_desc = cargo_scaffold::ScaffoldDescription::new(scaffold_ops)?;

    let params = scaffold_desc.fetch_parameters_value()?;

    scaffold_desc.scaffold_with_parameters(params)
}
