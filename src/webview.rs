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

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::error::Error;
use std::fs::{File, read_dir};
use std::io::Read;
use std::ops::Deref;
use std::rc::Rc;

use glib::ToVariant;
use gtk::{Inhibit, WidgetExt};
use libc::getpid;
use mg_settings::settings::Settings;
use url::Url;
use webkit2gtk::{self, CookiePersistentStorage, FindController, FindOptions, UserContentManager, UserScript, UserStyleSheet, WebContext, WebViewExt, FIND_OPTIONS_BACKWARDS, FIND_OPTIONS_CASE_INSENSITIVE, FIND_OPTIONS_WRAP_AROUND};
use webkit2gtk::UserContentInjectedFrames::AllFrames;
use webkit2gtk::UserScriptInjectionTime::End;
use webkit2gtk::UserStyleLevel::User;
use xdg::BaseDirectories;

use app::{AppResult, APP_NAME};
use message_server::MessageServer;
use settings::{AppSettings, CookieAcceptPolicy};
use settings::AppSettingsVariant::{
    CookieAccept,
    HintChars,
    HomePage,
    WebkitAllowFileAccessFromFileUrls,
    WebkitAllowModalDialogs,
    WebkitAutoLoadImages,
    WebkitCursiveFontFamily,
    WebkitDefaultCharset,
    WebkitDefaultFontFamily,
    WebkitDefaultFontSize,
    WebkitDefaultMonospaceFontSize,
    WebkitDrawCompositingIndicators,
    WebkitEnableAccelerated2dCanvas,
    WebkitEnableCaretBrowsing,
    WebkitEnableDeveloperExtras,
    WebkitEnableDnsPrefetching,
    WebkitEnableFrameFlattening,
    WebkitEnableFullscreen,
    WebkitEnableHtml5Database,
    WebkitEnableHtml5LocalStorage,
    WebkitEnableHyperlinkAuditing,
    WebkitEnableJava,
    WebkitEnableJavascript,
    WebkitEnableMediaStream,
    WebkitEnableMediasource,
    WebkitEnableOfflineWebApplicationCache,
    WebkitEnablePageCache,
    WebkitEnablePlugins,
    WebkitEnablePrivateBrowsing,
    WebkitEnableResizableTextAreas,
    WebkitEnableSiteSpecificQuirks,
    WebkitEnableSmoothScrolling,
    WebkitEnableSpatialNavigation,
    WebkitEnableTabsToLinks,
    WebkitEnableWebaudio,
    WebkitEnableWebgl,
    WebkitEnableWriteConsoleMessagesToStdout,
    WebkitEnableXssAuditor,
    WebkitFantasyFontFamily,
    WebkitJavascriptCanAccessClipboard,
    WebkitJavascriptCanOpenWindowsAutomatically,
    WebkitLoadIconsIgnoringImageLoadSetting,
    WebkitMediaPlaybackAllowsInline,
    WebkitMediaPlaybackRequiresUserGesture,
    WebkitMinimumFontSize,
    WebkitMonospaceFontFamily,
    WebkitPictographFontFamily,
    WebkitPrintBackgrounds,
    WebkitSansSerifFontFamily,
    WebkitSerifFontFamily,
    WebkitUserAgent,
    WebkitZoomTextOnly,
};
use stylesheet::get_stylesheet_and_whitelist;

const SCROLL_LINE_VERTICAL: i32 = 40;

/// Webkit-based view.
pub struct WebView {
    find_controller: FindController,
    message_server: MessageServer,
    scrolled_callback: RefCell<Option<Rc<Box<Fn(i64)>>>>,
    search_backwards: Cell<bool>,
    view: webkit2gtk::WebView,
}

impl WebView {
    /// Create a new web view.
    pub fn new() -> Rc<Self> {
        let context = WebContext::get_default().unwrap();
        if cfg!(debug_assertions) {
            context.set_web_extensions_directory("titanium-web-extension/target/debug");
        }
        else {
            let install_path = env!("TITANIUM_EXTENSION_INSTALL_PATH");
            context.set_web_extensions_directory(install_path);
        }

        let pid = unsafe { getpid() };
        let server_name = format!("com.titanium.process{}", pid);
        let message_server = MessageServer::new(&server_name, "/com/titanium/WebExtensions");

        context.set_web_extensions_initialization_user_data(&server_name.to_variant());

        let view = webkit2gtk::WebView::new_with_context_and_user_content_manager(&context, &UserContentManager::new());

        let find_controller = view.get_find_controller().unwrap();

        let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
        let cookie_path = xdg_dirs.place_data_file("cookies")
            .expect("cannot create data directory");
        let cookie_manager = context.get_cookie_manager().unwrap();
        cookie_manager.set_persistent_storage(cookie_path.to_str().unwrap(), CookiePersistentStorage::Sqlite);

        let webview =
            Rc::new(WebView {
                find_controller: find_controller,
                message_server: message_server,
                scrolled_callback: RefCell::new(None),
                search_backwards: Cell::new(false),
                view: view,
            });

        {
            let webview = webview.clone();
            let view = webview.view.clone();
            view.connect_draw(move |_, _| {
                webview.emit_scrolled_event();
                Inhibit(false)
            });
        }

        webview
    }

    /// Activate the selected hint.
    pub fn activate_hint(&self) -> Result<bool, Box<Error>> {
        self.view.grab_focus();
        self.message_server.activate_hint()
            .map_err(From::from)
    }

    /// Activate the link in the selection
    pub fn activate_selection(&self) -> AppResult {
        self.message_server.activate_selection()?;
        Ok(())
    }

    /// Add the user scripts.
    pub fn add_scripts(&self) -> AppResult {
        if let Some(content_manager) = self.view.get_user_content_manager() {
            content_manager.remove_all_scripts();
            let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
            let script_path = xdg_dirs.place_config_file("scripts")?;
            for filename in read_dir(script_path)? {
                let mut file = File::open(filename?.path())?;
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                // TODO: support whitelist as a comment in the script.
                let script = UserScript::new(&content, AllFrames, End, &[], &[]);
                content_manager.add_script(&script);
            }
        }
        Ok(())
    }

    /// Add the user stylesheets.
    pub fn add_stylesheets(&self) -> AppResult {
        if let Some(content_manager) = self.view.get_user_content_manager() {
            content_manager.remove_all_style_sheets();
            let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
            let stylesheets_path = xdg_dirs.place_config_file("stylesheets")?;
            for filename in read_dir(stylesheets_path)? {
                let mut file = File::open(filename?.path())?;
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                let (stylesheet, stylesheet_whitelist) = get_stylesheet_and_whitelist(&content);
                let whitelist: Vec<_> = stylesheet_whitelist.iter().map(|url| url.as_ref()).collect();
                let stylesheet = UserStyleSheet::new(&stylesheet, AllFrames, User, &whitelist, &[]);
                content_manager.add_style_sheet(&stylesheet);
            }
        }
        Ok(())
    }

    /// Connect the scrolled event.
    pub fn connect_scrolled<F: Fn(i64) + 'static>(&self, callback: F) {
        *self.scrolled_callback.borrow_mut() = Some(Rc::new(Box::new(callback)));
    }

    /// Emit the scrolled event.
    pub fn emit_scrolled_event(&self) {
        if let Some(ref callback) = *self.scrolled_callback.borrow() {
            let callback = callback.clone();
            if let Ok(scroll_percentage) = self.message_server.get_scroll_percentage() {
                callback(scroll_percentage);
            }
        }
    }

    /// Send a key to the web process to process with the current hints.
    pub fn enter_hint_key(&self, key_char: char) -> Result<bool, Box<Error>> {
        self.message_server.enter_hint_key(key_char)
            .map_err(From::from)
    }

    /// Clear the current search.
    pub fn finish_search(&self) {
        self.search("");
        self.find_controller.search_finish();
    }

    /// Follow a link.
    pub fn follow_link(&self, hint_chars: &str) -> AppResult {
        self.message_server.show_hint_on_links(hint_chars)?;
        Ok(())
    }

    /// Hide the hints.
    pub fn hide_hints(&self) -> AppResult {
        self.message_server.hide_hints()?;
        Ok(())
    }

    /// Open the specified URL.
    pub fn open(&self, url: &str) {
        let url: Cow<str> =
            if let Ok(_) = Url::parse(url) {
                url.into()
            }
            else {
                format!("http://{}", url).into()
            };
        self.view.load_uri(&url);
    }

    /// Scroll by the specified number of pixels.
    fn scroll(&self, pixels: i32) -> AppResult {
        self.message_server.scroll_by(pixels as i64)?;
        Ok(())
    }

    /// Scroll to the bottom of the page.
    pub fn scroll_bottom(&self) -> AppResult {
        self.message_server.scroll_bottom()?;
        Ok(())
    }

    /// Scroll down by one line.
    pub fn scroll_down_line(&self) -> AppResult {
        self.scroll(SCROLL_LINE_VERTICAL)
    }

    /// Scroll down by one half of page.
    pub fn scroll_down_half_page(&self) -> AppResult {
        let allocation = self.view.get_allocation();
        self.scroll(allocation.height / 2)
    }

    /// Scroll down by one page.
    pub fn scroll_down_page(&self) -> AppResult {
        let allocation = self.view.get_allocation();
        self.scroll(allocation.height)
    }

    /// Scroll to the top of the page.
    pub fn scroll_top(&self) -> AppResult {
        self.message_server.scroll_top()?;
        Ok(())
    }

    /// Scroll up by one line.
    pub fn scroll_up_line(&self) -> AppResult {
        self.scroll(-SCROLL_LINE_VERTICAL)
    }

    /// Scroll up by one half of page.
    pub fn scroll_up_half_page(&self) -> AppResult {
        let allocation = self.view.get_allocation();
        self.scroll(-allocation.height / 2)
    }

    /// Scroll up by one page.
    pub fn scroll_up_page(&self) -> AppResult {
        let allocation = self.view.get_allocation();
        self.scroll(-allocation.height)
    }

    /// Search some text.
    pub fn search(&self, input: &str) {
        let default_options = FIND_OPTIONS_CASE_INSENSITIVE | FIND_OPTIONS_WRAP_AROUND;
        let other_options =
            if self.search_backwards.get() {
                FIND_OPTIONS_BACKWARDS
            }
            else {
                FindOptions::empty()
            };
        let options = default_options | other_options;
        self.find_controller.search("", options.bits(), ::std::u32::MAX); // Clear previous search.
        self.find_controller.search(input, options.bits(), ::std::u32::MAX);
    }

    /// Search the next occurence of the search text.
    pub fn search_next(&self) {
        if self.search_backwards.get() {
            self.find_controller.search_previous();
        }
        else {
            self.find_controller.search_next();
        }
    }

    /// Search the previous occurence of the search text.
    pub fn search_previous(&self) {
        if self.search_backwards.get() {
            self.find_controller.search_next();
        }
        else {
            self.find_controller.search_previous();
        }
    }

    /// Adjust the webkit settings.
    pub fn setting_changed(&self, setting: &<AppSettings as Settings>::Variant) {
        if let Some(settings) = self.view.get_settings() {
            match *setting {
                CookieAccept(ref value) => self.set_cookie_accept(value),
                HintChars(_) | HomePage(_) => (),
                WebkitAllowFileAccessFromFileUrls(value) =>
                    settings.set_allow_file_access_from_file_urls(value),
                WebkitAllowModalDialogs(value) =>
                    settings.set_allow_modal_dialogs(value),
                WebkitAutoLoadImages(value) =>
                    settings.set_auto_load_images(value),
                WebkitCursiveFontFamily(ref value) =>
                    settings.set_cursive_font_family(value),
                WebkitDefaultCharset(ref value) =>
                    settings.set_default_charset(value),
                WebkitDefaultFontFamily(ref value) =>
                    settings.set_default_font_family(value),
                WebkitDefaultFontSize(value) =>
                    settings.set_default_font_size(value as u32),
                WebkitDefaultMonospaceFontSize(value) =>
                    settings.set_default_monospace_font_size(value as u32),
                WebkitDrawCompositingIndicators(value) =>
                    settings.set_draw_compositing_indicators(value),
                WebkitEnableAccelerated2dCanvas(value) =>
                    settings.set_enable_accelerated_2d_canvas(value),
                WebkitEnableCaretBrowsing(value) =>
                    settings.set_enable_caret_browsing(value),
                WebkitEnableDeveloperExtras(value) =>
                    settings.set_enable_developer_extras(value),
                WebkitEnableDnsPrefetching(value) =>
                    settings.set_enable_dns_prefetching(value),
                WebkitEnableFrameFlattening(value) =>
                    settings.set_enable_frame_flattening(value),
                WebkitEnableFullscreen(value) =>
                    settings.set_enable_fullscreen(value),
                WebkitEnableHtml5Database(value) =>
                    settings.set_enable_html5_database(value),
                WebkitEnableHtml5LocalStorage(value) =>
                    settings.set_enable_html5_local_storage(value),
                WebkitEnableHyperlinkAuditing(value) =>
                    settings.set_enable_hyperlink_auditing(value),
                WebkitEnableJava(value) =>
                    settings.set_enable_java(value),
                WebkitEnableJavascript(value) =>
                    settings.set_enable_javascript(value),
                WebkitEnableMediaStream(value) =>
                    settings.set_enable_media_stream(value),
                WebkitEnableMediasource(value) =>
                    settings.set_enable_mediasource(value),
                WebkitEnableOfflineWebApplicationCache(value) =>
                    settings.set_enable_offline_web_application_cache(value),
                WebkitEnablePageCache(value) =>
                    settings.set_enable_page_cache(value),
                WebkitEnablePlugins(value) =>
                    settings.set_enable_plugins(value),
                WebkitEnablePrivateBrowsing(value) =>
                    settings.set_enable_private_browsing(value),
                WebkitEnableResizableTextAreas(value) =>
                    settings.set_enable_resizable_text_areas(value),
                WebkitEnableSiteSpecificQuirks(value) =>
                    settings.set_enable_site_specific_quirks(value),
                WebkitEnableSmoothScrolling(value) =>
                    settings.set_enable_smooth_scrolling(value),
                WebkitEnableSpatialNavigation(value) =>
                    settings.set_enable_spatial_navigation(value),
                WebkitEnableTabsToLinks(value) =>
                    settings.set_enable_tabs_to_links(value),
                WebkitEnableWebaudio(value) =>
                    settings.set_enable_webaudio(value),
                WebkitEnableWebgl(value) =>
                    settings.set_enable_webgl(value),
                WebkitEnableWriteConsoleMessagesToStdout(value) =>
                    settings.set_enable_write_console_messages_to_stdout(value),
                WebkitEnableXssAuditor(value) =>
                    settings.set_enable_xss_auditor(value),
                WebkitFantasyFontFamily(ref value) =>
                    settings.set_fantasy_font_family(value),
                WebkitJavascriptCanAccessClipboard(value) =>
                    settings.set_javascript_can_access_clipboard(value),
                WebkitJavascriptCanOpenWindowsAutomatically(value) =>
                    settings.set_javascript_can_open_windows_automatically(value),
                WebkitLoadIconsIgnoringImageLoadSetting(value) =>
                    settings.set_load_icons_ignoring_image_load_setting(value),
                WebkitMediaPlaybackAllowsInline(value) =>
                    settings.set_media_playback_allows_inline(value),
                WebkitMediaPlaybackRequiresUserGesture(value) =>
                    settings.set_media_playback_requires_user_gesture(value),
                WebkitMinimumFontSize(value) =>
                    settings.set_minimum_font_size(value as u32),
                WebkitMonospaceFontFamily(ref value) =>
                    settings.set_monospace_font_family(value),
                WebkitPictographFontFamily(ref value) =>
                    settings.set_pictograph_font_family(value),
                WebkitPrintBackgrounds(value) =>
                    settings.set_print_backgrounds(value),
                WebkitSansSerifFontFamily(ref value) =>
                    settings.set_sans_serif_font_family(value),
                WebkitSerifFontFamily(ref value) =>
                    settings.set_serif_font_family(value),
                WebkitUserAgent(ref value) =>
                    settings.set_user_agent(Some(value)),
                WebkitZoomTextOnly(value) =>
                    settings.set_zoom_text_only(value),
            }
        }
    }

    /// Set the cookie accept policy.
    fn set_cookie_accept(&self, cookie_accept: &CookieAcceptPolicy) {
        let cookie_manager = self.view.get_context()
            .and_then(|context| context.get_cookie_manager());
        if let Some(cookie_manager) = cookie_manager {
            cookie_manager.set_accept_policy(cookie_accept.to_webkit());
        }
    }

    /// Set whether the search is backward or not.
    pub fn set_search_backward(&self, backward: bool) {
        self.search_backwards.set(backward);
    }

    /// Show the web inspector.
    pub fn show_inspector(&self) {
        static mut SHOWN: bool = false;
        if let Some(inspector) = self.view.get_inspector() {
            inspector.connect_attach(|inspector| {
                unsafe {
                    if !SHOWN {
                        inspector.detach();
                        SHOWN = true;
                        return true;
                    }
                    SHOWN = true;
                }
                false
            });
            inspector.connect_closed(|_| {
                unsafe {
                    SHOWN = false;
                }
            });
            inspector.show();
        }
    }

    /// Zoom in.
    pub fn zoom_in(&self) -> i32 {
        let level = self.view.get_zoom_level();
        self.view.set_zoom_level(level + 0.1);
        (self.view.get_zoom_level() * 100.0) as i32
    }

    /// Zoom back to 100%.
    pub fn zoom_normal(&self) -> i32 {
        self.view.set_zoom_level(1.0);
        100
    }

    /// Zoom out.
    pub fn zoom_out(&self) -> i32 {
        let level = self.view.get_zoom_level();
        self.view.set_zoom_level(level - 0.1);
        (self.view.get_zoom_level() * 100.0) as i32
    }
}

is_widget!(WebView, view);

impl Deref for WebView {
    type Target = webkit2gtk::WebView;

    fn deref(&self) -> &webkit2gtk::WebView {
        &self.view
    }
}
