use clap::CommandFactory;

fn main() {
    let markdown = clap_markdown::help_markdown::<hook0_cli::Cli>();

    let mut output = String::new();
    let mut skip = true; // skip until first subcommand section
    let mut in_toc = false;

    for line in markdown.lines() {
        // Skip the h1 title
        if line.starts_with("# ") && !line.starts_with("## ") {
            continue;
        }

        // Skip the TOC block
        if line.starts_with("**Command Overview:**") {
            in_toc = true;
            continue;
        }
        if in_toc {
            if line.starts_with("* [") || line.is_empty() {
                continue;
            }
            in_toc = false;
        }

        // Skip the root `## hook0` section (no subcommand)
        if line == "## `hook0`" {
            skip = true;
            continue;
        }

        // Start including at first subcommand section
        if line.starts_with("## `hook0 ") {
            skip = false;
        }

        if !skip {
            output.push_str(line);
            output.push('\n');
        }
    }

    // Remove trailing clap-markdown footer
    let output = output
        .trim_end()
        .lines()
        .take_while(|l| !l.starts_with("<hr/>"))
        .collect::<Vec<_>>()
        .join("\n");

    print!("{}", output.trim_end());
}
