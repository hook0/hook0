//! Optional TUI module for full-screen webhook listener interface.
//! Enabled via `--features tui`.

#[cfg(feature = "tui")]
mod app;
#[cfg(feature = "tui")]
mod echo_server;
#[cfg(feature = "tui")]
mod example_app;
#[cfg(feature = "tui")]
mod example_run;
#[cfg(feature = "tui")]
mod example_ui;
#[cfg(feature = "tui")]
mod json_editor;
#[cfg(feature = "tui")]
mod shared;
#[cfg(feature = "tui")]
mod ui;

#[cfg(feature = "tui")]
pub use app::run_tui;

#[cfg(feature = "tui")]
pub use example_run::run_example_tui;
