/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
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

use std::fs::{File, OpenOptions};
use std::io::{
    BufRead,
    BufReader,
    Read,
    Write,
};

use webkit2gtk::{
    Download,
    DownloadExt,
    URIResponseExt,
    WebViewExt,
};
use zip::ZipArchive;

use app::download::find_destination;
use app::Msg::HostfileDownloaded;
use download_list_view::Msg::DownloadRemove;
use errors::{Error, Result};
use super::App;
use urls::get_filename;

const WHITELIST: &[&str] = &[
    "localhost",
    "localhost.localdomain",
    "broadcasthost",
    "local",
];

impl App {
    pub fn adblock_update(&self) -> Result<()> {
        let hostfile = self.model.config_dir.data_file("hosts")?;
        let hostfile = hostfile.to_str()
            .ok_or_else(|| Error::new("Cannot get hostfile"))?;
        // Clear the current hosts file.
        File::create(hostfile)?;

        let urls = [
            "https://www.malwaredomainlist.com/hostslist/hosts.txt",
            "http://someonewhocares.org/hosts/hosts",
            "http://winhelp2002.mvps.org/hosts.zip",
            "http://malwaredomains.lehigh.edu/files/justdomains.zip",
            "https://pgl.yoyo.org/adservers/serverlist.php?hostformat=hosts&mimetype=plaintext",
        ];
        for url in &urls {
            if let Some(download) = self.webview.widget().download_uri(url) {
                let suggested_filename =
                    download.get_response()
                        .and_then(|response| response.get_suggested_filename())
                        .unwrap_or_else(|| get_filename(url).unwrap_or_default());
                if let Ok(destination) = find_destination(&self.model.config_dir, &suggested_filename) {
                    download.set_destination(&destination);
                    let destination = destination[7..].to_string(); // Remove file://
                    let down = download.clone();
                    connect!(self.model.relm, download, connect_finished(_),
                        HostfileDownloaded(destination.clone(), down.clone()));
                }
                else {
                    warn!("Cannot choose destination for file {}", suggested_filename);
                }
            }
            else {
                warn!("Cannot download file {}", url);
            }
        }

        Ok(())
    }

    pub fn process_hostfile(&self, filename: &str, download: Download) -> Result<()> {
        let just_domains = filename.contains("justdomains");
        info!("Processing host file {}", filename);
        let hostfile = self.model.config_dir.data_file("hosts")?;
        let hostfile = hostfile.to_str()
            .ok_or_else(|| Error::new("Cannot get hostfile"))?;
        if filename.ends_with(".zip") {
            let mut archive = ZipArchive::new(File::open(filename)?)?;
            let mut file =
                if filename.contains("/hosts") {
                    archive.by_name("HOSTS")?
                }
                else if just_domains {
                    archive.by_name("justdomains")?
                }
                else {
                    return Err(Error::new(&format!("Unknown host zip file: {}", filename)));
                };
            copy_file(hostfile, just_domains, &mut file)?;
        }
        else {
            let mut file = File::open(filename)?;
            copy_file(hostfile, just_domains, &mut file)?;
        }
        self.download_list_view.emit(DownloadRemove(download));
        Ok(())
    }
}

fn copy_file<R: Read>(hostfile: &str, just_domains: bool, file: &mut R) -> Result<()> {
    let mut buffer = [0; 4096];
    let mut hostfile = OpenOptions::new()
        .append(true)
        .write(true)
        .open(hostfile)?;
    if just_domains {
        while let Ok(size) = file.read(&mut buffer) {
            if size == 0 {
                break;
            }
            hostfile.write(&buffer[..size])?;
        }
    }
    else {
        let file = BufReader::new(file);
        for line in file.lines() {
            let mut line = line?;
            if let Some(index) = line.chars().position(|c| c == '#') {
                line.truncate(index);
            }
            let mut words = line.split_whitespace();
            words.next();
            if let Some(address) = words.next() {
                if !WHITELIST.contains(&address) {
                    writeln!(hostfile, "{}", address)?;
                }
            }
        }
    }
    Ok(())
}
