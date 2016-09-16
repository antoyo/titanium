/*
 * TODO: do not show an error when there is no config file.
 * TODO: map O to open with the current url.
 * TODO: only add space at the end when there is no argument (or don't add the space when there is
 * an URL?).
 * TODO: support modes (to avoid entering commands while typing in a text input).
 * TODO: support new window.
 * TODO: search (using FindController).
 * TODO: follow link.
 * TODO: write a webkit2 plugin to support scrolling.
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
 */

extern crate gtk;
extern crate mg;
#[macro_use]
extern crate mg_settings;
extern crate url;
extern crate webkit2;
extern crate xdg;

use mg::Application;
use mg_settings::Config;
use url::Url;
use webkit2::WebView;
use xdg::BaseDirectories;

use AppCommand::*;

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

fn main() {
    gtk::init().unwrap();

    let config = Config {
        mapping_modes: vec!["n".to_string()],
    };

    let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
    let config_path = xdg_dirs.place_config_file("config")
        .expect("cannot create configuration directory");

    let app = Application::new_with_config(config);
    app.use_dark_theme();
    let url_label = app.add_statusbar_item();

    if let Err(error) = app.parse_config(config_path) {
        app.error(error.description());
    }
    app.set_window_title(APP_NAME);

    let webview = WebView::new();
    webview.load_uri("https://duckduckgo.com/lite/");

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
                if let Ok(_) = Url::parse(&url) {
                    webview.load_uri(&url);
                }
                else {
                    webview.load_uri(&format!("http://{}", url));
                }
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
fn set_title(app: &Application<AppCommand>, webview: &WebView) {
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
