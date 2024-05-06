// Code for handling actual tutorials
//
// This code is invoked as
// `cargo xdp-tutorial add --name basic-03 <path-to-template> <tutorial-path>`

use std::path::Component;

use cargo_scaffold::Value;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author = "Abhijit Gadgil", version)]
pub(crate) struct AddCommand {
    /// Path to directory or a repository containing tutorial template.
    #[clap(name = "template-path")]
    template_path: std::path::PathBuf,

    /// Path to the directory containing tutorial that was created using `init` command.
    #[clap(name = "tutorial-path")]
    tutorial_path: std::path::PathBuf,

    /// Path inside the root of the repository, if the given template path is a GIT repository.
    #[clap(name = "repository-template-path", short = 'r', long = "path")]
    repository_template_path: Option<std::path::PathBuf>,

    /// Name to be given to the tutorial (defaults to the last element of the path of
    /// `template_path` or `repository_template_path`).
    #[clap(name = "name", short, long)]
    name: Option<String>,
}

pub(crate) fn do_add(cmd: AddCommand) -> anyhow::Result<()> {
    let ops = cargo_scaffold::Opts::builder(cmd.template_path.clone());

    let ops = if cmd.template_path.ends_with(".git") {
        if let Some(repository_template_path) = cmd.repository_template_path {
            ops.repository_template_path(repository_template_path)
        } else {
            ops
        }
    } else {
        ops
    };

    let project_name = cmd.tutorial_path.components().last();
    let ops = if let Some(project_name) = project_name {
        match project_name {
            Component::Normal(c) => {
                ops.project_name(c.to_str().expect("Invalid OS String").to_string())
            }
            _ => {
                return Err(anyhow::Error::msg("Invalid Target directory!"));
            }
        }
    } else {
        ops
    };

    let ops = ops.target_dir(cmd.tutorial_path).append(true);

    let scaffold_desc = cargo_scaffold::ScaffoldDescription::new(ops)?;

    let mut params = scaffold_desc.fetch_parameters_value()?;

    if let Some(name) = cmd.name {
        params.insert("tutorial_name".to_string(), Value::String(name));
    }

    scaffold_desc.scaffold_with_parameters(params)
}
