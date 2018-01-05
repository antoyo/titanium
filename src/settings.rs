/*
 * Copyright (c) 2016-2018 Boucher, Antoni <bouanto@zoho.com>
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

use webkit2gtk;

#[derive(Clone, Setting)]
pub enum CookieAcceptPolicy {
    #[default]
    Always,
    Never,
    NoThirdParty,
}

impl CookieAcceptPolicy {
    /// Convert the setting type to the webkit type.
    pub fn to_webkit(&self) -> webkit2gtk::CookieAcceptPolicy {
        match *self {
            CookieAcceptPolicy::Always => webkit2gtk::CookieAcceptPolicy::Always,
            CookieAcceptPolicy::Never => webkit2gtk::CookieAcceptPolicy::Never,
            CookieAcceptPolicy::NoThirdParty => webkit2gtk::CookieAcceptPolicy::NoThirdParty,
        }
    }
}

#[derive(Default, Settings)]
pub struct AppSettings {
    pub cookie_accept: CookieAcceptPolicy,
    pub hint_chars: String,
    pub home_page: String,
    pub webkit_allow_file_access_from_file_urls: bool,
    pub webkit_allow_modal_dialogs: bool,
    pub webkit_auto_load_images: bool,
    pub webkit_cursive_font_family: String,
    pub webkit_default_charset: String,
    pub webkit_default_font_family: String,
    pub webkit_default_font_size: i64,
    pub webkit_default_monospace_font_size: i64,
    pub webkit_draw_compositing_indicators: bool,
    pub webkit_enable_accelerated_2d_canvas: bool,
    pub webkit_enable_caret_browsing: bool,
    pub webkit_enable_developer_extras: bool,
    pub webkit_enable_dns_prefetching: bool,
    pub webkit_enable_frame_flattening: bool,
    pub webkit_enable_fullscreen: bool,
    pub webkit_enable_html5_database: bool,
    pub webkit_enable_html5_local_storage: bool,
    pub webkit_enable_hyperlink_auditing: bool,
    pub webkit_enable_java: bool,
    pub webkit_enable_javascript: bool,
    pub webkit_enable_media_stream: bool,
    pub webkit_enable_mediasource: bool,
    pub webkit_enable_offline_web_application_cache: bool,
    pub webkit_enable_page_cache: bool,
    pub webkit_enable_plugins: bool,
    pub webkit_enable_private_browsing: bool,
    pub webkit_enable_resizable_text_areas: bool,
    pub webkit_enable_site_specific_quirks: bool,
    pub webkit_enable_smooth_scrolling: bool,
    pub webkit_enable_spatial_navigation: bool,
    pub webkit_enable_tabs_to_links: bool,
    pub webkit_enable_webaudio: bool,
    pub webkit_enable_webgl: bool,
    pub webkit_enable_write_console_messages_to_stdout: bool,
    pub webkit_enable_xss_auditor: bool,
    pub webkit_fantasy_font_family: String,
    pub webkit_javascript_can_access_clipboard: bool,
    pub webkit_javascript_can_open_windows_automatically: bool,
    pub webkit_load_icons_ignoring_image_load_setting: bool,
    pub webkit_media_playback_allows_inline: bool,
    pub webkit_media_playback_requires_user_gesture: bool,
    pub webkit_minimum_font_size: i64,
    pub webkit_monospace_font_family: String,
    pub webkit_pictograph_font_family: String,
    pub webkit_print_backgrounds: bool,
    pub webkit_sans_serif_font_family: String,
    pub webkit_serif_font_family: String,
    pub webkit_user_agent: String,
    pub webkit_zoom_text_only: bool,
}
