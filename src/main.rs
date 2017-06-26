/*
 * Copyright (c) 2016-2017 Boucher, Antoni <bouanto@zoho.com>
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

/*
 * FIXME: cannot scroll on https://translate.google.com/translate?hl=fr&sl=es&tl=en&u=http%3A%2F%2Fblog.bltavares.com%2F2017%2F01%2F18%2Fexpressando_o_dominio_atraves_do_sistema_de_tipos%2F (find the closest node which can scroll: if more than one are found at the same level, use the largest)
 * TODO: add tests for the scrolling element.
 * TODO: allow to remove tags from bookmarks.
 *
 * TODO: switch to one UI process (and one message server).
 * TODO: save the current URLs of every window in case of a crash.
 * TODO: command to restore the last closed window.
 *
 * TODO: use connect_load_failed_with_tls_errors() to show the URL in red in case there's a TLS
 * error (test with https://www.pcwebshop.co.uk/)
 *
 * FIXME: using Escape in insert mode triggers Escape in the web page (in Scala doc: http://www.scala-lang.org/api/current/).
 * FIXME: sometimes, the page load percentage stays shown after the load is finished.
 *
 * TODO: remove unwrap() and expect() in dependencies (relm, mg).
 * TODO: remove every unwrap().
 * TODO: add a command to write a password into the focused text field.
 *
 * TODO: figure out why ErrorKind::Msg() is needed (cannot use "string".into() sometimes).
 *
 * FIXME: on http://ticki.github.io/blog/how-lz4-works/, clicking on the other article links at the
 * bottom redirect to a blank page.
 *
 * TODO: attempt to migrate to gecko.
 *
 * FIXME: file download should not navigate to a new page (see duckduckgo.com, because it uses a
 * redirection, this was working fine before).
 *
 * FIXME Error on exit: Io(Error { repr: Custom(Custom { kind: Other, error: Error { domain: 1537, code: 8, message: "Connexion ré-initialisée par le correspondant" } }) })
 *
 * TODO: method called by message should not take ownership of values.
 * TODO: add documentation for every method.
 * TODO: refactor to remove every use of callbacks in relm widgets.
 * FIXME: wrong scroll percentage on https://mail.gnome.org/archives/gtk-devel-list/2001-November/msg00204.html
 * FIXME: Invalid read of size 8 (see valgrind).
 *
 * TODO: use tokio-process to communicate with pass?
 *
 * TODO: shortcut to open the current URL's root.
 *
 * TODO: cli argument for minimal log level.
 * FIXME: negative zoom level.
 * FIXME: scrolling hides the info message.
 * FIXME: scrolling goes too far when zoomed in.
 * FIXME: hint not working on http://bibliotheque.ville.brossard.qc.ca/
 * TODO: modal dialog for authentication.
 * TODO: add a --redirect option or redirect command to bypass the adblocker.
 * TODO: show an error when a page is blocked by the adblocker.
 * FIXME: google is very slow.
 * FIXME: the browser (opened with window-new) is closed when the parent process is closed (mutt).
 * FIXME: the insert mode sometimes disable itself (using rofi-pass). For instance, on https://courrielweb.videotron.com/cw/legacyLoginResidentiel.action
 *
 * FIXME: hint on wrong location on the warning of https://zestedesavoir.com/tutoriels/1642/les-soins-non-urgents/#2-traiter-une-plaie
 * FIXME: hints on wrong locations on http://www.mensacanada.org/contact/ and on https://www.ralfj.de/blog/2017/06/06/MIR-semantics.html
 * FIXME: crash when attempting to open a PDF on Air Transat, Mon Dossier.
 * TODO: auto-delete tags.
 * FIXME: cookies are not synced between windows (cookies not reloaded in existing windows: use a thread and/or catch_unwind()).
 * FIXME: hitting the 's' key on https://developer.github.com/ scroll to the search bar.
 * FIXME: scrolling not working on http://www.freenom.com/en/termsandconditions.html
 * TODO: auto-detect static bars at the bottom/top of webpages to scroll less when one is present.
 *
 * FIXME: saving empty credentials on https://lichess4545.slack.com/
 *
 * TODO: sort bookmark completion with number of access (the most accessed URLs come first, then by
 * alphebetical order) and perhaps also by relevance (like the percentage of tags/words that
 * matches).
 * TODO: automatically propose tags when editting bookmark tags (fetch them from the webpage, <meta
 * property="og:article:tag"/> is a start).
 *
 * TODO: hint file input.
 * FIXME: open in new tab does not work in Github (https://github.com/rust-lang/rust/pull/37128).
 * Vimperator simulates ctrl-click to work around this.
 *
 * TODO: check if an extension process crashing causes issues in other extension process.
 * FIXME: missing hints on duckduckgo.com menu (caused by CSS3 transform).
 *
 * TODO: show an error when there are no hints.
 * FIXME: show hints for element with click event.
 *
 * TODO: handle the errors instead of unwrap().
 * TODO: #[default(value)] attribute for settings.
 *
 * TODO: show an error for request blocked by host blocker (instead of a white page).
 * TODO: add a command to do the redirections to avoid being blocked by the ad blocker.
 * FIXME: ctrl-/ should not trigger the mapping for /.
 * TODO: allow using Backspace to remove the last hint character.
 * TODO: hide HTML in title/bookmarks?
 * FIXME: the window sometimes does not hide when quitting: it hides when a new window is shown.
 * FIXME: hover does not always work (usherbrooke.ca) (perhaps trigger real click/hover mouse events in GTK+ instead of using DOM while still using the DOM focus function).
 * FIXME: an element visible but whose top-left corner is not shown wont get an hint.
 * FIXME: sometimes does not go to insert mode after focusing first input.
 * TODO: generate the default files from the code (for instance, from default settings) instead of
 * copying predefined files.
 * TODO: shortcut to open the selected (searched) word.
 * TODO: support CTRL-Z in input elements.
 * TODO: allow paste from selection clipboard (if the other is empty or with another shortcut?).
 * TODO: message when search fails (and when it wraps to the start/end).
 * TODO: hide the scrollbars?
 * FIXME: select dropdown can open in the other screen (webkit2gtk bug, move the cursor before clicking?).
 * TODO: unselect text when focusing a field.
 * TODO: add a passthrough mode.
 * TODO: add help text for commands and settings.
 * TODO: handle network errors.
 * TODO: support marks.
 * TODO: show a star next to the url of a bookmarked site.
 * TODO: find a way to recover accidently removed bookmarks (bookmark to readd bookmarks from a
 * stack of removed bookmarks? another shortcut which is harder to do C-S-d?).
 * TODO: warn when adding a bookmarks that has the same URL as another one, except with(out) a /.
 * TODO: preferred languages.
 * TODO: store cache.
 * TODO: add a command to delete history, …
 * TODO: NoScript.
 * TODO: open textarea in text editor.
 * TODO: add option to use light theme variant instead of dark variant.
 * TODO: private browsing.
 * TODO: soft scrolling (to avoid flickering for fixed elements, set_enable_smooth_scrolling).
 * TODO: do not search for the empty string, only disable the current search to allow continuing
 * the search on another page.
 * FIXME: do not show (or move) hints hidden by another element (branch button on GitHub).
 * TODO: show source.
 * TODO: prevent videos from autoplaying.
 * TODO: copier plugin (word, line, sentense, block, links…).
 * TODO: separate config options in section (like webkit.enable-java instead of webkit-enable-java).
 * TODO: i18n.
 * TODO: plugin to block modal JavaScript dialog (https://www.sitepoint.com/community/t/ie-hover-trigger/69968).
 * TODO: plugin to prevent a menu bar in a website to appear from scrolling up (ou keep fixed elements at the top)
 * (example: https://www.fastcoexist.com/3027876/millennials-dont-care-about-owning-cars-and-car-makers-cant-figure-out-why).
 * FIXME: trigger a GTK+ event to activate hints (this will fix clicking on a link hidden by
 * another element).
 * FIXME: popup not blocked on bnc.ca.
 * TODO: do not consider right-click open in new window as a popup.
 * TODO: delete the files opened (perhaps by placing them in a temporary directory).
 *
 * TODO: Rust-based plugin architecture (based on webkit web extensions).
 * TODO: command to update adblocker hosts file.
 * TODO: option to disable the adblocker.
 * TODO: block ads coming from websocket.
 * TODO: create a whitelist-based adblocker.
 * TODO: automatically detach the inspector when it is opened with "Inspect element".
 * TODO: remove the title bar of the inspector (window decorated property).
 * TODO: hide the hints when activating a hint.
 * TODO: in command and input mode, put the messages into a queue.
 * TODO: ask confirmation before submitting again the same form.
 * TODO: do not hard-code the extension directory: use the one provided by cargo.
 * TODO: find a way to install the titanium web extension library on cargo install.
 * TODO: activate insert mode after focusing a text element (disable insert mode when focus is lost).
 * TODO: show URL in title when the title is not available.
 * TODO: add command (;f) to focus frame.
 * TODO: add a validator for the file input (browse): check that a file is selected (and not a
 * directory), check that the input file exists.
 * FIXME: prompt slow to show (it seems to slow down when there are other events waiting: try
 * starting a download when the page is still loading).
 * FIXME: issues when multiple input are shown (they must be inserted in a queue and shown one at a
 * time, or perhaps just using a blocking input for popups will do it).
 *
 * TODO: add tests.
 *
 * FIXME: webview hides when resizing the screen (seems related to the web extension, or when the
 * page is not yet loaded, error: WebKitWebProcess: cairo-ft-font.c :669 : _cairo_ft_unscaled_font_lock_face:  l'assertion « !unscaled->from_face » a échoué.).
 */

//! Titanium is a webkit2 keyboard-driven web browser.

#![feature(proc_macro)]

#![warn(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
)]

extern crate cairo;
#[macro_use]
extern crate error_chain;
extern crate fg_uds;
extern crate futures;
extern crate futures_glib;
extern crate gdk;
extern crate glib;
extern crate gtk;
extern crate gumdrop;
#[macro_use]
extern crate gumdrop_derive;
#[cfg(test)]
extern crate libxdo;
#[macro_use]
extern crate log;
#[macro_use]
extern crate mg;
extern crate mg_settings;
#[macro_use]
extern crate mg_settings_macros;
extern crate number_prefix;
extern crate open;
extern crate password_store;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
extern crate rusqlite;
extern crate simplelog;
#[cfg(test)]
extern crate tempdir;
extern crate tempfile;
extern crate titanium_common;
extern crate tokio_io;
extern crate tokio_serde_bincode;
extern crate url;
extern crate webkit2gtk;
extern crate xdg;

mod app;
mod bookmarks;
mod clipboard;
mod commands;
mod completers;
mod config_dir;
mod download;
mod download_view;
mod download_list_view;
mod errors;
mod file;
mod message_server;
mod pass_manager;
mod popup_manager;
mod settings;
mod stylesheet;
mod urls;
mod webview;

use std::env::args;

use gumdrop::Options;
use log::LogLevel::Error;
use relm::Widget;
use simplelog::{Config, LogLevelFilter, TermLogger};

use app::{App, APP_NAME};

const INVALID_UTF8_ERROR: &str = "invalid utf-8 string";

#[derive(Debug, Default, Options)]
struct Args {
    #[options(help="The configuration directory.")]
    config: Option<String>,
    #[options(help="Print help message.")]
    help: bool,
    #[options(help="Show the log messages.")]
    log: bool,
    #[options(free)]
    url: Vec<String>,
}

fn main() {
    gtk::init().unwrap();

    let args: Vec<_> = args().collect();

    let mut args = match Args::parse_args_default(&args[1..]) {
        Ok(options) => options,
        Err(error) => {
            println!("{}: {}", APP_NAME, error);
            println!();
            println!("{}", Args::usage());
            return;
        },
    };

    if args.help {
        println!("Usage: {} [OPTIONS] [ARGUMENTS]", APP_NAME);
        println!();
        println!("{}", Args::usage());
    }
    else {
        if args.log {
            let config = Config {
                time: Some(Error),
                level: Some(Error),
                target: None,
                location: None,
            };
            TermLogger::init(LogLevelFilter::max(), config).unwrap();
        }

        let url =
            if !args.url.is_empty() {
                // TODO: open the other URLs in new windows?
                Some(args.url.remove(0))
            }
            else {
                None
            };

        App::run((url, args.config)).unwrap();
    }
}
