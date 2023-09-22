use anyhow::Error;
use clap::{ArgGroup, Args, Command};
use clap_complete::Shell;

#[derive(Args)]
#[command(group = ArgGroup::new("shell").required(true).multiple(false))]
pub struct CliArgs {
    /// For bash shell
    #[arg(short, long, group = "shell")]
    bash: bool,
    /// For zsh shell
    #[arg(short, long, group = "shell")]
    zsh: bool,
    /// For fish shell
    #[arg(short, long, group = "shell")]
    fish: bool,
}

pub fn print_completions(args: CliArgs, clap_app: &mut Command) -> Result<(), Error> {
    let shell = if args.bash {
        Shell::Bash
    } else if args.zsh {
        Shell::Zsh
    } else if args.fish {
        Shell::Fish
    } else {
        panic!("No shell argument")
    };
    let app_name = clap_app.get_name().to_string();
    clap_complete::generate(shell, clap_app, app_name, &mut std::io::stdout());
    Ok(())
}
