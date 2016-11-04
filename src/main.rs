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
 * TODO: file watcher to sync bookmarks between windows.
 *
 * FIXME: log macros not working.
 *
 * TODO: add tests.
 * TODO: create a declarative GUI library to avoid having all these RefCells.
 *
 * TODO: password fillers.
 *
 * TODO: check if an extension process crashing causes issues in other extension process.
 * FIXME: quitting the parent process (newsbeuter) seems to crash the extension process.
 * FIXME: panic when clicking the link at the bottom of developpeur.cool.
 * FIXME: missing hints on duckduckgo.com menu (caused by CSS3 transform).
 *
 * FIXME: open in new tab does not work in Github (https://github.com/rust-lang/rust/pull/37128).
 *
 * TODO: show an error when there are no hints.
 * FIXME: show hints for element with click event.
 * FIXME: a link on https://www.verywell.com/ear-pressure-pose-karnapidasana-3567089 cannot be
 * clicked (and many gobject critical error: g_object_ref assertion G_IS_OBJECT failed).
 * FIXME: go to insert mode for hints of multiple selection combo box.
 * FIXME: font color, family, size for hints (https://developer.mozilla.org/en-US/docs/Web/API/Window/open, https://www.chessbomb.com/).
 *
 * TODO: handle the errors instead of unwrap().
 * TODO: continue to parse the config files even when there are errors.
 * TODO: #[default(value)] attribute for settings.
 *
 * TODO: add help text for commands and settings.
 * TODO: allow to delete bookmarks in completion.
 * TODO: handle network errors.
 * TODO: support marks.
 * TODO: preferred languages.
 * TODO: store cache.
 * TODO: NoScript.
 * TODO: open textarea in text editor.
 * TODO: add option to use light theme variant instead of dark variant.
 * TODO: private browsing.
 * TODO: soft scrolling (to avoid flickering for fixed elements, set_enable_smooth_scrolling).
 * TODO: show source.
 * TODO: copier plugin (word, line, sentense, block, links…).
 * TODO: i18n.
 * TODO: block modal JavaScript dialog (https://www.sitepoint.com/community/t/ie-hover-trigger/69968).
 * TODO: handle ctrl-click.
 * TODO: do not consider right-click open in new window as a popup.
 * TODO: delete the files opened (perhaps by placing them in a temporary directory).
 *
 * TODO: block ads coming from websocket.
 * TODO: create a whitelist-based adblocker.
 * TODO: automatically detach the inspector when it is opened with "Inspect element".
 * TODO: remove the title bar of the inspector (window decorated property).
 * TODO: disable the tab key in the status bar input.
 * TODO: in command and input mode, put the messages into a queue.
 * TODO: ask confirmation before submitting again the same form.
 * TODO: do not hard-code the extension directory: use the one provided by cargo.
 * TODO: find a way to install the titanium web extension library on cargo install.
 * TODO: activate insert mode after focusing a text element.
 * FIXME: prompt slow to show (it seems to slow down when there are other events waiting: try
 * starting a download when the page is still loading).
 * FIXME: issues when multiple input are shown (they must be inserted in a queue and shown one at a
 * time, or perhaps just using a blocking input for popups will do it).
 *
 * FIXME: some dbus calls timeout (seems to be caused by the click method since it triggers an
 * action in the application which is waiting for the answer of the call).
 * FIXME: webview hides when resizing the screen (seems related to the web extension, or when the
 * page is not yet loaded, error: WebKitWebProcess: cairo-ft-font.c :669 : _cairo_ft_unscaled_font_lock_face:  l'assertion « !unscaled->from_face » a échoué.).
 */

//! Titanium is a webkit2 keyboard-driven web browser.

#![feature(proc_macro)]
#![warn(missing_docs)]

#[macro_use]
extern crate gdbus;
extern crate docopt;
extern crate gdk;
extern crate gio_sys;
extern crate glib;
extern crate glib_sys;
extern crate gtk;
extern crate gtk_sys;
extern crate libc;
#[macro_use]
extern crate mg;
#[macro_use]
extern crate mg_settings;
#[macro_use]
extern crate mg_settings_macros;
extern crate number_prefix;
extern crate open;
extern crate regex;
extern crate rustc_serialize;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate simplelog;
extern crate tempfile;
extern crate url;
extern crate webkit2gtk;
extern crate xdg;

mod app;
mod bookmarks;
mod clipboard;
mod commands;
mod completers;
mod dialogs;
mod download_view;
mod download_list_view;
mod file;
mod glib_user_dir;
mod message_server;
mod popup_manager;
mod settings;
mod stylesheet;
mod urls;
mod webview;

use docopt::Docopt;
use simplelog::TermLogger;
use simplelog::LogLevelFilter;

use app::App;

const USAGE: &'static str = "
Titanium web browser.

Usage:
    titanium [<url>] [--log]

Options:
    --log   Show the log messages.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_url: Option<String>,
    flag_log: bool,
}

fn main() {
    gtk::init().unwrap();

    let args: Args = Docopt::new(USAGE)
        .and_then(|decoder| decoder.decode())
        .unwrap_or_else(|error| error.exit());

    if args.flag_log {
        TermLogger::init(LogLevelFilter::max()).unwrap();
    }

    let _app = App::new(args.arg_url);

    gtk::main();
}
