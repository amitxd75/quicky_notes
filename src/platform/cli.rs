//! Command-line argument parsing for Quicky Notes.
//!
//! Supports opening files and directories directly from the terminal, e.g.:
//! - `quicky [file_path]`
//! - `quicky [folder_path]`
//! - `quicky -n` / `--new`
//! - `quicky -h` / `--help`
//! - `quicky -v` / `--version`

use std::path::PathBuf;

/// Action parsed from command-line arguments.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CliArgs {
    /// Files specified on the command line to open as tabs.
    pub files: Vec<PathBuf>,

    /// Directory specified on the command line to open as a folder workspace.
    pub folder: Option<PathBuf>,

    /// Whether a new blank note tab was requested explicitly (`-n` / `--new`).
    pub new_tab: bool,

    /// Whether `--help` or `-h` was requested.
    pub show_help: bool,

    /// Whether `--version` or `-v` was requested.
    pub show_version: bool,
}

impl CliArgs {
    /// Parses arguments from iterator of strings (excluding program name).
    pub fn parse<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let mut cli = Self::default();
        let mut iter = args.into_iter();

        while let Some(arg_item) = iter.next() {
            let arg = arg_item.as_ref();
            match arg {
                "-h" | "--help" => {
                    cli.show_help = true;
                }
                "-v" | "--version" => {
                    cli.show_version = true;
                }
                "-n" | "--new" => {
                    cli.new_tab = true;
                }
                "-F" | "--folder" => {
                    if let Some(next_arg) = iter.next() {
                        let path = PathBuf::from(next_arg.as_ref());
                        cli.folder = Some(path);
                    }
                }
                _ => {
                    let trimmed = arg.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if trimmed.starts_with('-') {
                        // Unrecognized flag; continue gracefully
                        continue;
                    }
                    // Filter out unexpanded desktop entry field codes (%F, %f, %U, %u, etc.)
                    if trimmed.starts_with('%') || trimmed.starts_with("\"%") {
                        continue;
                    }

                    // Strip file:// URI scheme if passed from file managers
                    let raw_path = if let Some(stripped) = trimmed.strip_prefix("file://") {
                        crate::ui::drag_drop::url_decode(stripped)
                    } else {
                        trimmed.to_string()
                    };

                    let path = PathBuf::from(raw_path);
                    if path.is_dir() {
                        if cli.folder.is_none() {
                            cli.folder = Some(path);
                        }
                    } else {
                        cli.files.push(path);
                    }
                }
            }
        }

        cli
    }

    /// Prints standard help message to stdout.
    pub fn print_help() {
        println!(
            r#"Quicky Notes - Floating Glassmorphism Note Widget & Editor

USAGE:
    quicky [OPTIONS] [PATH]...

ARGUMENTS:
    [PATH]...             One or more file paths or a directory path to open.

OPTIONS:
    -n, --new             Create a new blank note tab on launch
    -F, --folder <DIR>    Open specified directory as a folder workspace
    -h, --help            Print help information
    -v, --version         Print version information

EXAMPLES:
    quicky notes.md               Open or link notes.md
    quicky src/                   Open src/ directory in folder explorer
    quicky file1.rs file2.txt     Open multiple files into separate tabs
    quicky -n                     Launch with a clean new tab
"#
        );
    }

    /// Prints version message to stdout.
    pub fn print_version() {
        println!("quicky_notes {}", env!("CARGO_PKG_VERSION"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_empty() {
        let cli = CliArgs::parse(Vec::<String>::new());
        assert!(!cli.show_help);
        assert!(!cli.show_version);
        assert!(!cli.new_tab);
        assert!(cli.files.is_empty());
        assert!(cli.folder.is_none());
    }

    #[test]
    fn test_cli_parse_flags() {
        let cli_h = CliArgs::parse(["-h"]);
        assert!(cli_h.show_help);

        let cli_v = CliArgs::parse(["--version"]);
        assert!(cli_v.show_version);

        let cli_n = CliArgs::parse(["-n"]);
        assert!(cli_n.new_tab);
    }

    #[test]
    fn test_cli_parse_files_and_folder() {
        let args = ["file1.txt", "file2.md"];
        let cli = CliArgs::parse(args);
        assert_eq!(cli.files.len(), 2);
        assert_eq!(cli.files[0], PathBuf::from("file1.txt"));
        assert_eq!(cli.files[1], PathBuf::from("file2.md"));
    }

    #[test]
    fn test_cli_parse_explicit_folder_flag() {
        let args = ["--folder", "/tmp/my_workspace", "note.md"];
        let cli = CliArgs::parse(args);
        assert_eq!(cli.folder, Some(PathBuf::from("/tmp/my_workspace")));
        assert_eq!(cli.files.len(), 1);
        assert_eq!(cli.files[0], PathBuf::from("note.md"));
    }
}
