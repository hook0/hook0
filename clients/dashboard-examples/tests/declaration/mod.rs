//! A manifest written to put in front of the reader, so that a test about one entry is not a
//! test about the six others.

use std::path::PathBuf;

/// A manifest declaring everything a language has to, so that a test changing one entry is a test
/// about that entry rather than about the six others.
///
/// Written to a directory of its own under the temporary one, named after the running process:
/// several suites share that directory, and a file named `dashboard.toml` in it would be a file
/// several of them are writing at once.
pub struct Declaration {
    directory: PathBuf,
    pub display_name: String,
    pub usage_share: String,
    pub usage_source: String,
    pub proof: String,
    pub proves: String,
    /// The keys saying what puts the examples under the job, written out as they stand so that a
    /// case about that answer can leave them out or write them twice over.
    pub reach: String,
    /// What else a reader needs, written out as it stands and left empty by default: the key is the
    /// one a manifest may omit, so the manifest a case starts from is one that omits it.
    pub snippet_also_needs: String,
    pub label_separator: String,
    pub escape: String,
}

impl Declaration {
    pub fn new(named: &str) -> Declaration {
        let directory = std::env::temp_dir().join(format!(
            "hook0-dashboard-examples-{}-{named}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).expect("the temporary directory is not writable");
        Declaration {
            directory,
            display_name: "Example".to_owned(),
            usage_share: "1.0".to_owned(),
            usage_source: "# Stack Overflow Developer Survey 2025".to_owned(),
            proof: "compiled".to_owned(),
            proves: "built against clients/example by `build` in `clients.example.check`"
                .to_owned(),
            reach: "examples_named_in = \"clients/example/build.toml\"\n\
                    examples_named_by = [\"sources = [\\\"examples\\\"]\"]"
                .to_owned(),
            snippet_also_needs: String::new(),
            label_separator: ",\\n".to_owned(),
            escape: "[['\\', '\\\\'], ['\"', '\\\"']]".to_owned(),
        }
    }

    /// The file, written, so that reading it is reading a manifest rather than a fabrication of one.
    pub fn written(&self) -> PathBuf {
        let path = self
            .directory
            .join(hook0_dashboard_examples::manifest::FILE);
        std::fs::write(
            &path,
            format!(
                "display_name = \"{}\"\nproof = \"{}\"\nproves = \"{}\"\n{}\n{}\nusage_share = {}\n\
                 label_separator = \"{}\"\n{}\n\n[string]\nopen = '\"'\nclose = '\"'\nescape = {}\n",
                self.display_name,
                self.proof,
                self.proves,
                self.reach,
                self.usage_source,
                self.usage_share,
                self.label_separator,
                self.snippet_also_needs,
                self.escape
            ),
        )
        .expect("the manifest is not writable");
        path
    }
}

impl Drop for Declaration {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}
