use std::cmp::Ordering::{Greater, Less};
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use mg::completion::{Completer, CompletionCell, CompletionResult};
use mg::completion::Column::{self, Expand};

use file::download_dir;

/// A file completer.
pub struct FileCompleter {
    current_directory: PathBuf,
}

impl FileCompleter {
    /// Create a new file completer.
    pub fn new() -> Self {
        let path = Path::new(&download_dir()).to_path_buf();
        FileCompleter {
            current_directory: path,
        }
    }
}

impl Completer for FileCompleter {
    fn columns(&self) -> Vec<Column> {
        vec![Expand]
    }

    fn complete_result(&self, value: &str) -> String {
        let absolute_path = self.current_directory.join(value);
        // Remove the trailing slash in the completion to avoid updating the completions for a new
        // directory when selecting a directory.
        // This means the user needs to type the slash to trigger the completion of the new
        // directory.
        absolute_path.to_str().unwrap().trim_right_matches('/').to_string()
    }

    fn completions(&mut self, input: &str) -> Vec<CompletionResult> {
        let mut paths = vec![];
        let input_path = Path::new(input).to_path_buf();
        // If the input ends with /, complete within this directory.
        // Otherwise, complete the files from the parent directory.
        let path =
            if !input.ends_with('/') {
                input_path.parent()
                    .map(Path::to_path_buf)
                    .unwrap_or(input_path)
            }
            else {
                input_path
            };
        self.current_directory = path.clone();
        if let Ok(entries) = read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    let matched = {
                        let absolute_path_string = path.to_str().unwrap();
                        let path_string = path.file_name().unwrap().to_str().unwrap();
                        // Do not show hidden files (starting with dot).
                        !path_string.starts_with('.') && absolute_path_string.starts_with(input)
                    };
                    if matched {
                        paths.push(path);
                    }
                }
            }
        }
        // Sort directories first, then sort by name.
        paths.sort_by(|path1, path2| {
            match (path1.is_dir(), path2.is_dir()) {
                (true, false) => Less,
                (false, true) => Greater,
                _ => path1.cmp(path2),
            }
        });
        paths.iter()
            .map(|path| {
                let filename = path.file_name().unwrap().to_str().unwrap();
                if path.is_dir() {
                    let mut filename = filename.to_string();
                    filename.push('/');
                    CompletionResult::from_cells(&[&CompletionCell::new(&filename).foreground("#33FF33")])
                }
                else {
                    CompletionResult::new(&[&filename.to_string()])
                }
            })
            .collect()
    }
}