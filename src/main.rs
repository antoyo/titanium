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
 * TODO: show an error when there are no hints.
 * FIXME: missing hints on duckduckgo.com menu (caused by CSS3 transform).
 * FIXME: show hints for element with click event.
 * FIXME: a link on https://www.verywell.com/ear-pressure-pose-karnapidasana-3567089 cannot be
 * clicked (and many gobject critical error: g_object_ref assertion G_IS_OBJECT failed).
 * FIXME: go to insert mode for hints of multiple selection combo box.
 *
 * FIXME: can only be launched from the terminal.
 * TODO: continue to parse the config files even when there are errors.
 * TODO: #[default(value)] attribute for settings.
 * TODO: settings completion.
 * TODO: add shortcuts like Tab, Shift-Tab, Ctrl-P and Ctrl-N to command completion.
 * TODO: download manager.
 * TODO: support bookmarks with tags (shortcut to delete bookmark by current URL).
 * TODO: open completions.
 * TODO: open file (instead of download).
 * TODO: follow in new window.
 * TODO: adblock.
 * TODO: copy/paste URLs.
 * TODO: handle network errors.
 * TODO: support marks.
 * TODO: preferred languages.
 * TODO: store cache.
 * TODO: NoScript.
 * TODO: open textarea in text editor.
 * TODO: add option to use light theme variant instead of dark variant.
 * TODO: add content to the default config file.
 * TODO: private browsing.
 * TODO: soft scrolling (to avoid flickering for fixed elements, set_enable_smooth_scrolling).
 * TODO: copier plugin (word, line, sentense, block, links…).
 * TODO: i18n.
 *
 * TODO: remove the title bar of the inspector (window decorated property).
 * TODO: disable the tab key in the status bar input.
 * TODO: in command and input mode, put the messages into a queue.
 * TODO: ask confirmation before submitting again the same form.
 * TODO: do not hard-code the extension directory: use the one provided by cargo.
 * TODO: find a way to install the titanium web extension library on cargo install.
 * TODO: activate insert mode after focusing a text element.
 * FIXME: prompt slow to show.
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
extern crate gtk;
extern crate gtk_sys;
extern crate libc;
#[macro_use]
extern crate mg;
#[macro_use]
extern crate mg_settings;
#[macro_use]
extern crate mg_settings_macros;
extern crate rustc_serialize;
extern crate simplelog;
extern crate url;
extern crate webkit2gtk;
extern crate xdg;

mod app;
mod commands;
mod message_server;
mod popup_manager;
mod settings;
mod stylesheet;
mod urls;
mod webview;

use docopt::Docopt;
use simplelog::TermLogger;
use simplelog::LogLevelFilter::{self, Off};

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

    let filter_level =
        if args.flag_log {
            LogLevelFilter::max()
        }
        else {
            Off
        };
    TermLogger::init(filter_level).unwrap();

    let _app = App::new(args.arg_url);

    gtk::main();
}
