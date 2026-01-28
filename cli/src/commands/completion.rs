use anyhow::Result;
use clap::{Args, CommandFactory};
use clap_complete::{generate, Shell};
use std::io;

#[derive(Args, Debug)]
pub struct CompletionArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,
}

pub fn execute(args: &CompletionArgs) -> Result<()> {
    let mut cmd = crate::Cli::command();
    let name = cmd.get_name().to_string();

    generate(args.shell, &mut cmd, name, &mut io::stdout());

    eprintln!("\n# Installation instructions:");
    match args.shell {
        Shell::Bash => {
            eprintln!("# Add to ~/.bashrc or ~/.bash_profile:");
            eprintln!("# eval \"$(hook0 completion bash)\"");
            eprintln!("#");
            eprintln!("# Or save to a file:");
            eprintln!("# hook0 completion bash > ~/.local/share/bash-completion/completions/hook0");
        }
        Shell::Zsh => {
            eprintln!("# Add to ~/.zshrc:");
            eprintln!("# eval \"$(hook0 completion zsh)\"");
            eprintln!("#");
            eprintln!("# Or save to a file (ensure fpath includes this directory):");
            eprintln!("# hook0 completion zsh > ~/.zfunc/_hook0");
        }
        Shell::Fish => {
            eprintln!("# Save to fish completions directory:");
            eprintln!("# hook0 completion fish > ~/.config/fish/completions/hook0.fish");
        }
        Shell::PowerShell => {
            eprintln!("# Add to your PowerShell profile:");
            eprintln!("# hook0 completion powershell | Out-String | Invoke-Expression");
        }
        Shell::Elvish => {
            eprintln!("# Add to ~/.elvish/rc.elv:");
            eprintln!("# eval (hook0 completion elvish | slurp)");
        }
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_variants() {
        // Just verify that all shell variants are supported
        let shells = vec![
            Shell::Bash,
            Shell::Zsh,
            Shell::Fish,
            Shell::PowerShell,
            Shell::Elvish,
        ];

        for shell in shells {
            // Should not panic
            let _ = format!("{:?}", shell);
        }
    }
}
