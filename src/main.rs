/*
 * Copyright (c) 2016 Boucher, Antoni <bouanto@zoho.com>
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
 * FIXME: negative zoom level.
 * FIXME: running `cargo run -- fsf.org` opens about:blank.
 * FIXME: using Escape in insert mode triggers Escape in the web page (in Scala doc).
 * FIXME: the insert mode sometimes disable itself (using rofi-pass). For instance, on https://courrielweb.videotron.com/cw/legacyLoginResidentiel.action
 * FIXME: I can set tags on URLs that are not bookmarked.
 * FIXME: hints on wrong locations on http://www.mensacanada.org/contact/.
 * FIXME: follow open in a new window after a follow-win.
 * FIXME: crash when attempting to open a PDF on Air Transat, Mon Dossier.
 * FIXME: web process crash on print (follow onclick="javascript:print").
 * TODO: auto-delete tags.
 * TODO: allow to remove tags from bookmarks.
 * FIXME: cookies are not synced between windows (cookies not reloaded in existing windows: use a thread and/or catch_unwind()).
 *
 * TODO: unlock the password store when loading a password.
 * FIXME: loading credentials does not work.
 * FIXME: saving empty credentials on https://lichess4545.slack.com/
 *
 * TODO: switch to one UI process (and one DBus server to see if it resolves the timeout issues).
 * TODO: save the current URLs of every window in case of a crash.
 * TODO: command to restore the last closed window.
 *
 * TODO: sort bookmark completion with number of access (the most accessed URLs come first, then by
 * alphebetical order).
 * TODO: automatically propose tags when editting bookmark tags (fetch them from the webpage, <meta
 * property="og:article:tag"/> is a start).
 *
 * TODO: use lifetimes to ensure the pointers live long enough for the connect!() macro.
 *
 * TODO: hint file input.
 * FIXME: open in new tab does not work in Github (https://github.com/rust-lang/rust/pull/37128).
 * Vimperator simulates ctrl-click to work around this.
 *
 * TODO: Create a gdbus binding that works similar to elm subscriptions.
 *
 * TODO: check if an extension process crashing causes issues in other extension process.
 * FIXME: missing hints on duckduckgo.com menu (caused by CSS3 transform).
 *
 * TODO: show an error when there are no hints.
 * FIXME: show hints for element with click event.
 * FIXME: a link on https://www.verywell.com/ear-pressure-pose-karnapidasana-3567089 cannot be
 * clicked (and many gobject critical error: g_object_ref assertion G_IS_OBJECT failed).
 *
 * TODO: handle the errors instead of unwrap().
 * TODO: continue to parse the config files even when there are errors.
 * TODO: #[default(value)] attribute for settings.
 *
 * TODO: show an error for request blocked by host blocker.
 * TODO: add a command to do the redirections to avoid being blocked by the ad blocker.
 * FIXME: ctrl-/ should not trigger the mapping for /.
 * TODO: allow using Backspace to remove the last hint character.
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
 * FIXME: titanium seems slower than other browsers.
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
 * TODO: use a custom error type (wrapping the other errors) instead of Box<Error>.
 * TODO: show source.
 * TODO: prevent videos from autoplaying.
 * TODO: copier plugin (word, line, sentense, block, links…).
 * TODO: separate config options in section (like webkit.enable-java instead of webkit-enable-java).
 * TODO: i18n.
 * TODO: plugin to block modal JavaScript dialog (https://www.sitepoint.com/community/t/ie-hover-trigger/69968).
 * FIXME: trigger a GTK+ event to activate hints (this will fix clicking on a link hidden by
 * another element).
 * TODO: do not consider right-click open in new window as a popup.
 * TODO: delete the files opened (perhaps by placing them in a temporary directory).
 *
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
 * FIXME: cannot scroll on http://dudzik.co/digress-into-development/syntax-off/.
 * FIXME: prompt slow to show (it seems to slow down when there are other events waiting: try
 * starting a download when the page is still loading).
 * FIXME: issues when multiple input are shown (they must be inserted in a queue and shown one at a
 * time, or perhaps just using a blocking input for popups will do it).
 *
 * TODO: add tests.
 *
 * FIXME: some dbus calls timeout (seems to be caused by the click method since it triggers an
 * action in the application which is waiting for the answer of the call).
 * FIXME: webview hides when resizing the screen (seems related to the web extension, or when the
 * page is not yet loaded, error: WebKitWebProcess: cairo-ft-font.c :669 : _cairo_ft_unscaled_font_lock_face:  l'assertion « !unscaled->from_face » a échoué.).
 */

//! Titanium is a webkit2 keyboard-driven web browser.

#![warn(missing_docs)]

extern crate cairo;
#[macro_use]
extern crate gdbus;
extern crate docopt;
extern crate gdk;
extern crate gio_sys;
extern crate glib;
extern crate glib_sys;
extern crate gtk;
extern crate gtk_sys;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[cfg(test)]
extern crate libxdo;
#[macro_use]
extern crate log;
#[macro_use]
extern crate mg;
#[macro_use]
extern crate mg_settings;
#[macro_use]
extern crate mg_settings_macros;
extern crate number_prefix;
extern crate open;
extern crate rusqlite;
extern crate rustc_serialize;
#[macro_use]
extern crate secret;
extern crate simplelog;
#[cfg(test)]
extern crate tempdir;
extern crate tempfile;
extern crate titanium_common;
extern crate url;
extern crate webkit2gtk;
extern crate xdg;

mod app;
mod bookmarks;
mod clipboard;
mod commands;
mod completers;
mod config_dir;
mod dialogs;
mod download;
mod download_view;
mod download_list_view;
mod file;
mod glib_ext;
mod message_server;
mod pass_manager;
mod popup_manager;
mod settings;
mod stylesheet;
mod urls;
mod webview;

use docopt::Docopt;
use log::LogLevel::Error;
use simplelog::{Config, LogLevelFilter, TermLogger};

use app::App;

const USAGE: &'static str = "
Titanium web browser.

Usage:
    titanium [<url>] [--config=<dir>] [--log]

Options:
    --config=<dir>  The configuration directory.
    --log           Show the log messages.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_url: Option<String>,
    flag_config: Option<String>,
    flag_log: bool,
}

fn main() {
    gtk::init().unwrap();

    let args: Args = Docopt::new(USAGE)
        .and_then(|decoder| decoder.decode())
        .unwrap_or_else(|error| error.exit());

    if args.flag_log {
        let config = Config {
            time: Some(Error),
            level: Some(Error),
            target: None,
            location: None,
        };
        TermLogger::init(LogLevelFilter::max(), config).unwrap();
    }

    let _app = App::new(args.arg_url, args.flag_config);

    gtk::main();
}
