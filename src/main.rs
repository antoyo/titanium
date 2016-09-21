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
 * TODO: clear search (Searchnext still works after hitting Escape).
 * FIXME: webview hides when resizing the screen (seems related to the web extension, or when the
 * page is not yet loaded, error: WebKitWebProcess: cairo-ft-font.c :669 : _cairo_ft_unscaled_font_lock_face:  l'assertion « !unscaled->from_face » a échoué.).
 * TODO: show error if communication is not working.
 * TODO: follow link.
 * TODO: activate insert mode after focusing a text element.
 * TODO: support bookmarks with tags.
 * TODO: settings (third-party cookies).
 * TODO: download manager.
 * TODO: open file (instead of download).
 * TODO: adblock.
 * TODO: command/open completions.
 * TODO: copy/paste URLs.
 * TODO: handle network errors.
 * TODO: activate selected links with Enter.
 * TODO: support marks.
 * TODO: preferred languages.
 * TODO: store cache.
 * TODO: log console message to stdout.
 * TODO: NoScript.
 * TODO: open textarea in text editor.
 * TODO: non-modal javascript alert, prompt and confirm.
 * TODO: add option to use light theme variant instead of dark variant.
 * TODO: add content to the default config file.
 * TODO: switch from dbus to gdbus.
 */

extern crate dbus;
#[macro_use]
extern crate dbus_macros;
extern crate docopt;
extern crate gdk;
extern crate glib;
extern crate gtk;
extern crate gtk_sys;
extern crate libc;
#[macro_use]
extern crate mg;
#[macro_use]
extern crate mg_settings;
extern crate rustc_serialize;
extern crate url;
extern crate webkit2;
extern crate xdg;

mod app;
mod message_server;
mod webview;

use docopt::Docopt;

use app::App;

const USAGE: &'static str = "
Titanium web browser.

Usage:
    titanium [<url>]
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_url: Option<String>,
}

fn main() {
    gtk::init().unwrap();

    let args: Args = Docopt::new(USAGE)
        .and_then(|decoder| decoder.decode())
        .unwrap_or_else(|error| error.exit());

    let _app = App::new(args.arg_url);

    gtk::main();
}
