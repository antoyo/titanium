/*
 * Copyright (c) 2019 Boucher, Antoni <bouanto@zoho.com>
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

use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use glib::Cast;
use webkit2gtk::{
    self, GeolocationPermissionRequest, NotificationPermissionRequest, UserMediaPermissionRequest,
    UserMediaPermissionRequestExt,
};

use app::App;
use config_dir::ConfigDir;
use errors::Result;
use file;
use urls::get_base_url;

use self::PermissionDescription::*;

pub enum Permission {
    Always,
    Never,
}

#[derive(Hash, PartialEq, Eq)]
enum PermissionDescription {
    Geolocation,
    Microphone,
    Notification,
    Webcam,
}

impl PermissionDescription {
    fn from(string: &str) -> Option<Self> {
        match string {
            "geolocation" => Some(Geolocation),
            "microphone" => Some(Microphone),
            "notification" => Some(Notification),
            "webcam" => Some(Webcam),
            _ => None,
        }
    }

    fn from_request(request: &webkit2gtk::PermissionRequest) -> Option<Self> {
        if request.is::<GeolocationPermissionRequest>() {
            Some(Geolocation)
        } else if request.is::<NotificationPermissionRequest>() {
            Some(Notification)
        } else if let Ok(media_permission) =
            request.clone().downcast::<UserMediaPermissionRequest>()
        {
            if media_permission.get_property_is_for_video_device() {
                Some(Webcam)
            } else {
                Some(Microphone)
            }
        } else {
            None
        }
    }
}

impl Display for PermissionDescription {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let string = match *self {
            Geolocation => "geolocation",
            Microphone => "microphone",
            Notification => "notification",
            Webcam => "webcam",
        };
        write!(formatter, "{}", string)
    }
}

/// Manager to know whether a permission should be always or never be accepted.
pub struct PermissionManager {
    blacklisted_urls: HashSet<(String, PermissionDescription)>,
    blacklist_path: PathBuf,
    whitelisted_urls: HashSet<(String, PermissionDescription)>,
    whitelist_path: PathBuf,
}

impl PermissionManager {
    /// Create a new permission manager.
    pub fn new(whitelist_path: PathBuf, blacklist_path: PathBuf) -> Self {
        PermissionManager {
            blacklisted_urls: HashSet::new(),
            blacklist_path,
            whitelisted_urls: HashSet::new(),
            whitelist_path,
        }
    }

    /// Blacklist the specified url.
    pub fn blacklist(
        &mut self,
        url: &str,
        permission: &webkit2gtk::PermissionRequest,
    ) -> Result<()> {
        match (
            get_base_url(url),
            PermissionDescription::from_request(permission),
        ) {
            (Some(url), Some(permission)) => {
                self.blacklisted_urls.insert((url.to_string(), permission));
                self.save_blacklist()
            }
            _ => {
                warn!("Not blacklisting {}", url);
                Ok(())
            }
        }
    }

    /// Check if the specified url is blacklisted.
    pub fn is_blacklisted(&self, url: &str, permission: &webkit2gtk::PermissionRequest) -> bool {
        match PermissionDescription::from_request(permission) {
            Some(permission) => self
                .blacklisted_urls
                .contains(&(get_base_url(url).unwrap_or_else(String::new), permission)),
            None => false,
        }
    }

    /// Check if the specified url is whitelisted.
    pub fn is_whitelisted(&self, url: &str, permission: &webkit2gtk::PermissionRequest) -> bool {
        match PermissionDescription::from_request(permission) {
            Some(permission) => self
                .whitelisted_urls
                .contains(&(get_base_url(url).unwrap_or_else(String::new), permission)),
            None => false,
        }
    }

    /// Load the urls from the files.
    pub fn load(&mut self) -> Result<()> {
        self.blacklisted_urls = self.read_as_set(&self.blacklist_path)?;
        self.whitelisted_urls = self.read_as_set(&self.whitelist_path)?;
        Ok(())
    }

    /// Read a file as a HashSet where all lines are one entry in the set.
    fn read_as_set(&self, path: &PathBuf) -> Result<HashSet<(String, PermissionDescription)>> {
        let mut file = file::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let set = content
            .lines()
            .filter(|s| !s.is_empty())
            .filter_map(|s| {
                let mut words = s.split_whitespace();
                match (words.next(), words.next()) {
                    (Some(url), Some(permission)) => {
                        Some((url.to_string(), PermissionDescription::from(permission)?))
                    }
                    _ => None,
                }
            })
            .collect();
        Ok(set)
    }

    /// Save the list in the file specified by `path`.
    fn save(&self, path: &PathBuf, list: &HashSet<(String, PermissionDescription)>) -> Result<()> {
        let mut file = File::create(path)?;
        for (url, permission) in list {
            writeln!(file, "{} {}", url, permission)?;
        }
        Ok(())
    }

    /// Save the permission blacklist.
    fn save_blacklist(&self) -> Result<()> {
        self.save(&self.blacklist_path, &self.blacklisted_urls)
    }

    /// Save the permission whitelist.
    fn save_whitelist(&self) -> Result<()> {
        self.save(&self.whitelist_path, &self.whitelisted_urls)
    }

    /// Whitelist the specified url.
    pub fn whitelist(
        &mut self,
        url: &str,
        permission: &webkit2gtk::PermissionRequest,
    ) -> Result<()> {
        match (
            get_base_url(url),
            PermissionDescription::from_request(permission),
        ) {
            (Some(url), Some(permission)) => {
                self.whitelisted_urls.insert((url.to_string(), permission));
                self.save_whitelist()
            }
            _ => {
                warn!("Not whitelisting {}", url);
                Ok(())
            }
        }
    }
}

/// Create a permission manager if the blacklist/whitelist paths can be created.
pub fn create_permission_manager(config_dir: &ConfigDir) -> Option<PermissionManager> {
    if let (Ok(whitelist_path), Ok(blacklist_path)) = App::permission_path(config_dir) {
        Some(PermissionManager::new(whitelist_path, blacklist_path))
    } else {
        None
    }
}
