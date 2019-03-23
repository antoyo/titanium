/*
 * Copyright (c) 2016-2018 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use std::io;
use std::path::PathBuf;

use xdg::{BaseDirectories, BaseDirectoriesError};

use self::ConfigDirOption::{Path, Xdg};
use app::APP_NAME;

#[derive(Clone)]
enum ConfigDirOption {
    Path(PathBuf),
    Xdg(BaseDirectories),
}

/// Configuration directory manager.
/// If `config_dir` is `None`, the XDG directory will be used.
#[derive(Clone)]
pub struct ConfigDir {
    dir: ConfigDirOption,
}

impl ConfigDir {
    pub fn new(config_dir: &Option<String>) -> Result<Self, BaseDirectoriesError> {
        let dir = if let Some(ref config_dir) = *config_dir {
            Path(PathBuf::from(config_dir))
        } else {
            Xdg(BaseDirectories::with_prefix(APP_NAME)?)
        };
        Ok(ConfigDir { dir: dir })
    }

    /// Get a path to the config file.
    pub fn config_file(&self, filename: &str) -> io::Result<PathBuf> {
        match self.dir {
            Path(ref path) => {
                let mut path = path.clone();
                path.push("config");
                path.push(filename);
                Ok(path)
            }
            Xdg(ref xdg) => xdg.place_config_file(filename),
        }
    }

    /// Get the home configuration directory.
    pub fn config_home(&self) -> PathBuf {
        match self.dir {
            Path(ref path) => {
                let mut path = path.clone();
                path.push("config");
                path
            }
            Xdg(ref xdg) => xdg.get_config_home(),
        }
    }

    /// Get a path to the data file.
    pub fn data_file(&self, filename: &str) -> io::Result<PathBuf> {
        match self.dir {
            Path(ref path) => {
                let mut path = path.clone();
                path.push("data");
                path.push(filename);
                Ok(path)
            }
            Xdg(ref xdg) => xdg.place_data_file(filename),
        }
    }

    /// Get the home data directory.
    pub fn data_home(&self) -> PathBuf {
        match self.dir {
            Path(ref path) => {
                let mut path = path.clone();
                path.push("data");
                path
            }
            Xdg(ref xdg) => xdg.get_data_home(),
        }
    }
}
