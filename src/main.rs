/*
 * Copyright (c) 2016-2020 Boucher, Antoni <bouanto@zoho.com>
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
 * TODO: add cosmetic filter support in adblocker: https://github.com/dudik/blockit
 * (doc: https://github.com/brave/adblock-rust/issues/152#issuecomment-771259069)
 * (that probably requires a MutationObserver: https://github.com/brave/adblock-rust/issues/152#issuecomment-771046487)
 *
 * FIXME: the adblocker seems to block good URL like https://blog.mozilla.org/security/2021/01/26/supercookie-protections/
 * TODO: add cosmetic filter support in adblocker: https://github.com/dudik/blockit
 * (doc: https://github.com/brave/adblock-rust/issues/152#issuecomment-771259069
 * (that probably requires a MutationObserver: https://github.com/brave/adblock-rust/issues/152#issuecomment-771046487)
 *
 * FIXME: auto-login on https://www.iga.net/fr/mon_compte/se_connecter
 *
 * FIXME: many popups in alibaba.com/google.com.
 * TODO: Use HitTarget for right-click to automatically allow these popups to open.
 *
 * FIXME: cannot enter username/password on https://us-redhat.icims.com/jobs/75775/software-engineer---llvm-%26--go-toolchain/login?loginOnly=1&redirect=job&hashed=-435800245
 * TODO: switch to https://github.com/anlumo/cef ?
 * FIXME: ads on https://www.webmd.com/mental-health/what-are-symptoms-ptsd
 *
 * TODO: shortcut to show the URL of a link.
 *
 * TODO: whitelist ads in hydro québec to allow login.
 *
 * TODO: disable youtube auto-play.
 *
 * TODO: update the easylist instead of the former hosts-based lists.
 *
 * TODO: this URL (https://smallbusiness.chron.com/hide-things-certain-people-facebook-29815.html)
 * freezes the window.
 *
 * TODO: implement the buster extension to automatically solve google recaptcha (https://addons.mozilla.org/fr/firefox/addon/buster-captcha-solver/).
 *
 * TODO: add shortcut to go to next input field?
 *
 * FIXME: pages full on links freeze when using follow.
 *
 * TODO: add shortcut to scroll by paragraph (like { and } in vim).
 *
 * FIXME: enabling webkit-mediasource breaks some youtube videos.
 * FIXME: downloading a file on a different partition does not work.
 * TODO: prevent screensaver when playing video.
 *
 * TODO: command to change user agent (with a set of default).
 *
 * TODO: command to input username?
 *
 * normal video: videoplayback?clen=26562312&itag=43&mime=video%2Fwebm&gir=yes&key=yt6&mv=m&c=web&ei=yE8ZW--bBoqH-QOR7ZjIAw&initcwndbps=867500&lmt=1516735696905984&mm=31,29&mn=sn-4pcgxovpgx-t0ae,sn-t0a7sn7d&id=o-AOM1kOpaVnUns1y_vIkM0Xxe-TssoxptbMiah5HPg
 * ads: videoplayback?ip=70.35.215.109&lmt=1526332905241335&itag=43&requiressl=yes&id=o-AICQD28nyUKvGo_q-p47C57Yq_MvY3o78bQ3yYn1nVFu&pcm2cms=yes&source=youtube&dur=0.000&key=yt6&mn=sn-4pcgxovpgx-t0ae,sn-t0a7sn7d&mm=31,29&pl=25&mv=m&mt=152838538
 *
 * FIXME: Submit login form does not work on Zoho and Grafana.
 * FIXME: URLs not opening in new window on http://www.canadiantire.ca/fr/outdoor-living/outdoor-power-equipment/lawn-mowers/electric-lawn-mowers.html?adlocation=LIT_Content_Category_ElectricMower_fr
 * * https://www.homedepot.ca/fr/accueil/categories/decor/meubles/mobilier-de-salle-de-sejour/tables-basses-et-tables-de-bout.html
 * * https://www.kijiji.ca/b-longueuil-rive-sud/table-basse/k0l1700279?dc=true
 *
 * TODO: block ads on YouTube: greasemonkey scripts to block youtube video ads
 * http://userscripts-mirror.org/scripts/review/173910
 * https://github.com/airodactyl/qutebrowser/commit/48eb50d7e
 * https://greasyfork.org/en/scripts/9932-youtube
 *
 * TODO: Add a shortcut to copy from selection clipboard in insert mode.
 * TODO: use String as error.
 *
 * TODO: might find this useful for for filler: https://webkitgtk.org/reference/webkit2gtk/stable/WebKitWebPage.html#WebKitWebPage-form-controls-associated
 *
 * FIXME: submit not working in bnc.ca login form.
 *
 * FIXME: cannot follow the clone button on GitHub anymore.
 *
 * FIXME: Hint on wrong position on: http://www.travbuddy.com/search_google.php?cx=001087027826441394888%3Ap6zx7j2vnt8&cof=FORID%3A9&q=test (now a scroll issue)
 * https://queue.acm.org/detail.cfm?id=3185224
 *
 * FIXME: hints on two lines on: https://area.autodesk.com/all/tutorials/stingray/?p=2
 *
 * FIXME: kill-win does not unfreeze the other web pages in the same web process.
 *
 * TODO: settings containing a list of websites that automatically open in private mode?
 *
 * FIXME: If titanium was opened by xdg-open (by e.g. zathura), the abstract domain socket can live
 * longer than what we want, preventing a new titanium from starting.
 * ~ ss -apx | rg titanium
 * u_str  LISTEN     11     128    @titanium-server 38583                 * 0
 * +users:(("zathura",pid=7755,fd=11),("xdg-open",pid=7637,fd=11))
 *
 * FIXME: cannot open html with from file:// url.
 *
 * FIXME: web process crash on a specific website (only the first time): might be caused by
 * webkit2gtk itself.
 *
 * FIXME: scroll broken on http://kcsrk.info/ocaml/multicore/2015/05/20/effects-multicore/
 *
 * TODO: shortcut to insert the password in insert mode?
 * TODO: shortcut to open in the same window.
 *
 * FIXME: slow startup (The update function was slow to execute for message NewApp: 1943ms).
 * Perhaps should init table and cleanup download folder in another thread.
 *
 * FIXME: Links not working on https://tecnocode.co.uk/2014/03/27/what-is-gmaincontext/ (target="_blank")
 * FIXME: message corrupt on LinkedIn (libsoup issue).
 *
 * TODO: exit insert mode after hitting Enter in text input.
 * TODO: default window size.
 *
 * FIXME: config file marks not created.
 *
 * TODO: save mark when focusing the first input.
 * TODO: save current scroll position in ' register before starting a search.
 *
 * FIXME: using Escape in insert mode triggers Escape in the web page (in Scala doc: http://www.scala-lang.org/api/current/).
 *
 * TODO: plugin for a read mode (remove all useless stuff in the page, only keep the text).
 *
 * FIXME: hitting the 's' key on https://developer.github.com/ scroll to the search bar.
 *
 * TODO: show an error when there are no hints.
 *
 * TODO: handle network errors.
 * TODO: show an error for request blocked by host blocker (instead of a white page).
 *
 * TODO: ask confirmation before submitting again the same form.
 *
 * TODO: do not consider right-click open in new window as a popup.
 *
 * TODO: add command to save credentials by encrypting the username.
 *
 * FIXME: issues when multiple input dialogs are shown (they must be inserted in a queue and shown one at a
 * time, or perhaps just using a blocking input for popups will do it).
 * TODO: in command and input mode, put the messages into a queue.
 *
 * TODO: option to disable the adblocker.
 * TODO: remove duplicates in the hostfile.
 *
 * TODO: remove the title bar of the inspector (window decorated property).
 *
 * TODO: do not search for the empty string, only disable the current search to allow continuing
 * the search on another page.
 *
 * TODO: unselect text when focusing a field.
 * TODO: support CTRL-Z in input elements.
 *
 * FIXME: ctrl-/ should not trigger the mapping for /.
 *
 * TODO: allow using Backspace to remove the last hint character.
 *
 * TODO: remove ads on DuckDuckGo Lite.
 *
 * TODO: Command to know which pages are in which process:
 * * to check whether the pages are distributed evenly between the processes.
 * * maybe to know if a process is stuck.
 *
 * TODO: hide HTML in title/bookmarks?
 *
 * TODO: message when search fails (and when it wraps to the start/end).
 *
 * TODO: show source.
 *
 * FIXME: scrolling hides the info message.
 * FIXME: scrolling goes too far when zoomed in.
 * FIXME: negative zoom level.
 *
 * TODO: modal dialog for authentication.
 *
 * TODO: add a command to delete history, …
 *
 * TODO: show the letters typed in follow mode.
 *
 * TODO: command to restore the last closed window.
 * TODO: command to open last deleted bookmark?
 *
 * TODO: rename the quit command to close.
 * TODO: add a close-all command?
 *
 * TODO: automatically add new settings in the config files (kind of insertion sort, splitted by
 * the command? Detect whether a keymap was changed. What about those that were deleted?).
 *
 * TODO: should the private context be cleaned up when all the private windows are closed?
 * TODO: should there be a new private context every time the command private-win-open is issued?
 *
 * TODO: ResetMarks.
 *
 * TODO: hide hovered link when the text entry is shown? (Or show the hovered URL instead of the
 * current URL?)
 * TODO: bigger text entry than URL label.
 *
 * FIXME: should not silently fail when an included file is missing.
 *
 * TODO: webkit_web_view_get_main_resource() to get source code
 * TODO: Show hints on elements with an ID (to be able to navigate to their anchor).
 *
 * FIXME: angular form needs the typing action to be done in order to submit: https://www.codingame.com/start
 *
 * FIXME: seems slower when running as normal user (and faster as root), so perhaps the config slow
 * it down. Looks like it is slowed down by the hard drive.
 *
 * TODO: plugin to hide disqus.
 *
 * TODO: downloading a non-existing file (http://download.microsoft.com/download/8/8/8/888f34b7-4f54-4f06-8dac-fa29b19f33dd/msxml3.msi) causes an error.
 *
 * TODO: might not require the syntax with (relm) if we emit the signal normally.
 *
 * TODO: shortcut to toggle between open and win-open.
 *
 * TODO: delete file if download is halted (when browser closes).
 *
 * FIXME: windows opened by JavaScript cannot be claused: probably need to set the settings
 * javascript_can_close_windows when the window was opened by JS.
 *
 * TODO: websites with login form (or credit card input) should be shown as insecure if not in
 * HTTPS.
 *
 * FIXME: scroll on
 * https://tutorial.ponylang.org/getting-started/how-it-works.html
 * https://www.fstar-lang.org/tutorial/
 * https://www.cnet.com/special-reports/mozilla-firefox-fights-back-against-google-chrome/
 * FIXME: scroll percentage wrong on:
 * file:///home/bouanto
 * http://www.expressionsofchange.org/reification-of-interaction/
 * https://www.snellman.net/blog/archive/2017-04-17-xxx-fixme/
 *
 * FIXME: too many redirections: https://www.cavendre.com/fr/annonce/login.php
 *
 * TODO: add command (;f) to change the active element.
 *
 * TODO: remove unwrap() and expect() in dependencies (relm, mg).
 * TODO: remove every unwrap().
 *
 * FIXME: on http://ticki.github.io/blog/how-lz4-works/, clicking on the other article links at the
 * bottom redirect to a blank page.
 *
 * TODO: attempt to migrate to gecko.
 *
 * FIXME: Error on exit: Io(Error { repr: Custom(Custom { kind: Other, error: Error { domain: 1537, code: 8, message: "Connexion ré-initialisée par le correspondant" } }) })
 * FIXME: Error on exit: Error sending IPC message: Relais brisé (pipe)
 *
 * TODO: method called by message should not take ownership of values.
 * TODO: add documentation for non-obvious code.
 * TODO: refactor to remove every use of callbacks in relm widgets.
 * FIXME: Invalid read of size 8 (see valgrind).
 *
 * TODO: show [<>] like vimperator to show whether we can go back/forward in the history?
 *
 * TODO: shortcut zz to center the selected text (from a search, for instance).
 * TODO: shortcuts like zt and zb (is there something similar in vim that is not related to the cursor?).
 *
 * TODO: use asynchronous communication with pass (to avoid blocking other windows)?
 *
 * FIXME: cannot scroll on https://translate.google.com/translate?hl=fr&sl=es&tl=en&u=http%3A%2F%2Fblog.bltavares.com%2F2017%2F01%2F18%2Fexpressando_o_dominio_atraves_do_sistema_de_tipos%2F (find the closest node which can scroll: if more than one are found at the same level, use the largest)
 * TODO: add tests for the scrolling element.
 *
 * TODO: cli argument for the abstract namespace of the unix domain socket.
 * TODO: cli argument for minimal log level.
 *
 * TODO: add a --redirect option or redirect command to bypass the adblocker.
 * TODO: add a command to do the redirections to avoid being blocked by the ad blocker.
 *
 * TODO: block cookie banner.
 * FIXME: the insert mode sometimes disable itself (using rofi-pass). For instance, on https://courrielweb.videotron.com/cw/legacyLoginResidentiel.action
 *
 * TODO: stop sending message to a web process after it crashed:
 * (.:1266): GLib-CRITICAL **: g_io_channel_write_chars: assertion 'channel->is_writeable' failed
 * (.:1266): GLib-CRITICAL **: g_source_modify_unix_fd: assertion 'g_slist_find (source->priv->fds, tag)' failed
 *
 * FIXME: crash when attempting to open a PDF on Air Transat, Mon Dossier.
 * TODO: auto-delete tags.
 * FIXME: scrolling not working on http://www.freenom.com/en/termsandconditions.html
 * TODO: auto-detect static bars at the bottom/top of webpages to scroll less when one is present.
 *
 * TODO: prevent from auto-downloading videos.
 *
 * FIXME: saving empty credentials on https://lichess4545.slack.com/
 *
 * TODO: shortcut to copy selected text (without being in insert mode).
 *
 * TODO: shortcut to (un)check all checkboxes in page (for email notification pages with many
 * checkboxes).
 *
 * TODO: sort bookmark completion with number of access (the most accessed URLs come first, then by
 * alphebetical order) and perhaps also by relevance (like the percentage of tags/words that
 * matches).
 * TODO: automatically propose tags when editting bookmark tags (fetch them from the webpage, <meta
 * property="og:article:tag"/> is a start).
 *
 * TODO: hint file input.
 * TODO: find a way to avoid having hints on top of each other.
 *
 * TODO: check if an extension process crashing causes issues in other extension process.
 *
 * FIXME: show hints for element with click event.
 *
 * TODO: #[default(value)] attribute for settings.
 *
 * FIXME: if search engine query contains #, it does not include it (and what follows it) in the
 * search form.
 *
 * TODO: feature to detect when there's a login page for a network.
 *
 * FIXME: wrong formatting on https://doc.rust-lang.org/stable/book/first-edition/testing.html#the-tests-directory
 *
 * TODO: feature to import bookmarks from .sqlite file.
 *
 * TODO: support creating bookmarks with:
 * https://developer.mozilla.org/en-US/Add-ons/WebExtensions/API/bookmarks/create
 *
 * FIXME: the window sometimes does not hide when quitting: it hides when a new window is shown.
 * FIXME: hover does not always work (usherbrooke.ca) (perhaps trigger real click/hover mouse events in GTK+ instead of using DOM while still using the DOM focus function).
 * FIXME: sometimes does not go to insert mode after focusing first input (youtube.com).
 * TODO: generate the default files from the code (for instance, from default settings) instead of
 * copying predefined files.
 * TODO: shortcut to open the selected (searched) word.
 * TODO: allow paste from selection clipboard (if the other is empty or with another shortcut?).
 * TODO: hide the scrollbars?
 * FIXME: select dropdown can open in the other screen (webkit2gtk bug, move the cursor before clicking?).
 * TODO: add a passthrough mode.
 * TODO: add help text for commands and settings.
 * TODO: show a star next to the url of a bookmarked site.
 * TODO: warn when adding a bookmarks that has the same URL as another one, except with(out) a /.
 * TODO: store cache.
 * TODO: NoScript.
 * TODO: open textarea in text editor.
 * TODO: add option to use light theme variant instead of dark variant.
 * TODO: soft scrolling (to avoid flickering for fixed elements, set_enable_smooth_scrolling).
 * FIXME: do not show (or move) hints hidden by another element (branch button on GitHub).
 * TODO: prevent videos from autoplaying.
 * TODO: copier plugin (word, line, sentense, block, links…).
 * TODO: separate config options in section (like webkit.enable-java instead of webkit-enable-java).
 * TODO: i18n.
 * TODO: plugin to block modal JavaScript dialog (https://www.sitepoint.com/community/t/ie-hover-trigger/69968).
 * TODO: plugin to prevent a menu bar in a website to appear from scrolling up (ou keep fixed elements at the top)
 * (example: https://www.fastcoexist.com/3027876/millennials-dont-care-about-owning-cars-and-car-makers-cant-figure-out-why).
 * FIXME: popup not blocked on bnc.ca.
 * TODO: delete the files opened (perhaps by placing them in a temporary directory).
 *
 * TODO: allow to search bookmarks not containing a tag or containing only the specified tags.
 *
 * TODO: Rust-based plugin architecture (based on webkit web extensions).
 * TODO: block ads coming from websocket.
 * TODO: create a whitelist-based adblocker.
 * TODO: hide the hints when activating a hint.
 * TODO: do not hard-code the extension directory: use the one provided by cargo.
 * TODO: find a way to install the titanium web extension library on cargo install.
 * TODO: activate insert mode after focusing a text element (disable insert mode when focus is lost).
 * TODO: block coin miner (take blacklist from https://github.com/keraf/NoCoin).
 * TODO: add a validator for the file input (browse): check that a file is selected (and not a
 * directory), check that the input file exists.
 * FIXME: prompt slow to show (it seems to slow down when there are other events waiting: try
 * starting a download when the page is still loading).
 *
 * TODO: add tests.
 */

//! Titanium is a webkit2 keyboard-driven web browser.

#![allow(deprecated)]
#![warn(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
)]

extern crate cairo;
extern crate gdk;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate gumdrop;
#[cfg(test)]
extern crate libxdo;
#[macro_use]
extern crate log;
extern crate log_panics;
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
#[macro_use]
extern crate relm_derive;
extern crate rusqlite;
extern crate simplelog;
extern crate syslog;
extern crate tempfile;
extern crate titanium_common;
extern crate url;
extern crate webkit2gtk;
extern crate xdg;
extern crate zip;

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
mod permission_manager;
mod popup_manager;
mod settings;
mod stylesheet;
mod urls;
mod webview;

use std::{env::args, collections::BTreeSet};

use gio::{prelude::ApplicationExtManual, File, traits::{ApplicationExt, FileExt}};
use gtk::{Application, traits::GtkApplicationExt};
use gumdrop::Options;
use log::Level::Error;
use relm::{EventStream, Relm, Update, UpdateNew, execute, init, Component};
use simplelog::{Config, LevelFilter, TermLogger};
use syslog::Facility;

use app::{App, APP_NAME};
use config_dir::ConfigDir;
use message_server::{create_message_server, MessageServer, Msg::NewApp, Privacy};
use webview::WebView;

const INVALID_UTF8_ERROR: &str = "invalid utf-8 string";

/// The GTK app name.
#[cfg(not(debug_assertions))]
pub const GTK_APP_NAME: &str = "com.titanium-browser";
/// A different GTK app name is used in debug mode for easier debugging.
#[cfg(debug_assertions)]
pub const GTK_APP_NAME: &str = "com.titanium-browser.debug";

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
    let args: Vec<_> = args().collect();

    /*let args = match Args::parse_args_default(&args[1..]) {
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
    else {*/
        //init_logging(args.log);

        let application = Application::new(Some(GTK_APP_NAME), gio::ApplicationFlags::HANDLES_OPEN);

        let _app = execute::<RelmApp>(application.clone());

        application.run(/*&args*/);
        //let _message_server = create_message_server(args.url, args.config);
    //}
}

struct RelmApp {
    model: Model,
}

struct Model {
    application: Application,
    message_server: Option<EventStream<<MessageServer as Update>::Msg>>,
    wins: Vec<Component<App>>,
}

#[derive(Msg)]
enum Msg {
    Activate,
    Open(Vec<File>),
}

impl Update for RelmApp {
    type Model = Model;
    type ModelParam = Application;
    type Msg = Msg;

    fn model(_: &Relm<Self>, application: Self::ModelParam) -> Model {
        Model {
            application,
            message_server: None,
            wins: vec![],
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        connect!(self.model.application, connect_activate(app), relm, {
            // NOTE: we need to increase the refcount of app because we create the window
            // asynchronously.
            // TODO: perhaps it won't be needed anymore when we remove the client/server
            // architecture.
            // TODO: that probably requires calling release() later.
            app.hold();
            Msg::Activate
        });
        connect!(self.model.application, connect_open(_, files, _), relm, Msg::Open(files.to_vec()));
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Activate => {
                self.model.message_server = Some(create_message_server(vec![], None));
            },
            Msg::Open(files) => {
                for file in files {
                    println!("URI: {}", file.uri());
                    if let Some(ref message_server) = self.model.message_server {
                        message_server.stream().emit(NewApp(Some(file.uri().to_string()), Privacy::Normal));
                    }
                }
            },
        }
    }
}

impl UpdateNew for RelmApp {
    fn new(_relm: &Relm<Self>, model: Self::Model) -> Self {
        RelmApp {
            model,
        }
    }
}

fn init_logging(log_to_term: bool) {
    if log_to_term {
        let config = Config {
            time: Some(Error),
            level: Some(Error),
            target: None,
            location: None,
            time_format: None,
        };
        TermLogger::init(LevelFilter::max(), config).unwrap();
    }
    else {
        syslog::init_unix(Facility::LOG_USER, LevelFilter::max()).unwrap();
    }
    log_panics::init();
}
