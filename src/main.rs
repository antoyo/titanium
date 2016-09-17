/*
 * TODO: map O to open with the current url.
 * TODO: only add space at the end when there is no argument (or don't add the space when there is
 * an URL?).
 * TODO: support modes (to avoid entering commands while typing in a text input).
 * TODO: support new window.
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

mod webview;

use std::fs::OpenOptions;

use docopt::Docopt;
use mg::Application;
use mg_settings::Config;
use xdg::BaseDirectories;

use AppCommand::*;
use webview::WebView;

macro_rules! unwrap_or_show_error {
    ($app:expr, $error:expr) => {
        if let Err(error) = $error {
            $app.error(&error.to_string());
        }
    };
}

commands!(AppCommand {
    Back,
    Forward,
    Open(String),
    Quit,
    Reload,
    Reloadbypasscache,
    Stop,
});

const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
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

    let config = Config {
        mapping_modes: vec!["n".to_string()],
    };

    let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
    let config_path = xdg_dirs.place_config_file("config")
        .expect("cannot create configuration directory");

    let app = Application::new_with_config(config);
    app.use_dark_theme();
    let url_label = app.add_statusbar_item();

    unwrap_or_show_error!(app, OpenOptions::new().create(true).write(true).open(&config_path));
    unwrap_or_show_error!(app, app.parse_config(config_path));

    app.set_window_title(APP_NAME);

    let home_page = "https://duckduckgo.com/lite/".to_string();
    let url = args.arg_url.unwrap_or(home_page);
    let webview = WebView::new();
    webview.open(&url);

    {
        let app = app.clone();
        webview.connect_load_changed(move |webview, _load_event| {
            if let Some(url) = webview.get_uri() {
                url_label.set_text(&url);
            }

            set_title(&app, webview);
        });
    }
    {
        let app = app.clone();
        webview.connect_resource_load_started(move |webview, _resource, _uri_request| {
            set_title(&app, webview);
        });
    }
    app.set_view(&webview);

    app.connect_command(move |command| {
        match command {
            Back => webview.go_back(),
            Forward => webview.go_forward(),
            Open(url) => {
                webview.open(&url);
            },
            Quit => gtk::main_quit(),
            Reload => webview.reload(),
            Reloadbypasscache => webview.reload_bypass_cache(),
            Stop => webview.stop_loading(),
        }
    });

    gtk::main();
}

/// Set the title of the window as the progress and the web page title.
fn set_title(app: &Application<AppCommand>, webview: &webkit2::WebView) {
    let progress = (webview.get_estimated_load_progress() * 100.0) as i32;
    if let Some(title) = webview.get_title() {
        if progress == 100 {
            app.set_window_title(&format!("{} - {}", title, APP_NAME));
        }
        else {
            app.set_window_title(&format!("[{}%] {} - {}", progress, title, APP_NAME));
        }
    }
}
