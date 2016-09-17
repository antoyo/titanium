/*
 * TODO: only add space at the end when there is no argument (or don't add the space when there is
 * an URL?).
 * TODO: support modes (to avoid entering commands while typing in a text input).
 * TODO: support new window.
 * TODO: support special commands.
 * TODO: search (using FindController).
 * TODO: follow link.
 * TODO: write a webkit2 plugin to support scrolling.
 * TODO: settings.
 * TODO: cookie.
 * TODO: download manager.
 * TODO: open file (instead of download).
 * TODO: support bookmarks with tags.
 * TODO: adblock.
 * TODO: command/open completions.
 * TODO: copy/paste URLs.
 * TODO: handle network errors.
 * TODO: NoScript.
 * TODO: open textarea in text editor.
 * TODO: non-modal javascript alert, prompt and confirm.
 * TODO: add option to use light theme variant instead of dark variant.
 * TODO: add content to the default config file.
 */

extern crate docopt;
extern crate glib;
extern crate gtk;
extern crate gtk_sys;
#[macro_use]
extern crate mg;
#[macro_use]
extern crate mg_settings;
extern crate rustc_serialize;
extern crate url;
extern crate webkit2;
extern crate xdg;

mod app;
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
