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

use super::WebView;
use settings::AppSettingsVariant::{
    CookieAccept, HintChars, HomePage, WebkitAllowFileAccessFromFileUrls, WebkitAllowModalDialogs,
    WebkitAutoLoadImages, WebkitCursiveFontFamily, WebkitDefaultCharset, WebkitDefaultFontFamily,
    WebkitDefaultFontSize, WebkitDefaultMonospaceFontSize, WebkitDrawCompositingIndicators,
    WebkitEnableAccelerated2dCanvas, WebkitEnableCaretBrowsing, WebkitEnableDeveloperExtras,
    WebkitEnableDnsPrefetching, WebkitEnableFrameFlattening, WebkitEnableFullscreen,
    WebkitEnableHtml5Database, WebkitEnableHtml5LocalStorage, WebkitEnableHyperlinkAuditing,
    WebkitEnableJava, WebkitEnableJavascript, WebkitEnableMediaStream, WebkitEnableMediasource,
    WebkitEnableOfflineWebApplicationCache, WebkitEnablePageCache, WebkitEnablePlugins,
    WebkitEnablePrivateBrowsing, WebkitEnableResizableTextAreas, WebkitEnableSiteSpecificQuirks,
    WebkitEnableSmoothScrolling, WebkitEnableSpatialNavigation, WebkitEnableTabsToLinks,
    WebkitEnableWebaudio, WebkitEnableWebgl, WebkitEnableWriteConsoleMessagesToStdout,
    WebkitEnableXssAuditor, WebkitFantasyFontFamily, WebkitJavascriptCanAccessClipboard,
    WebkitJavascriptCanOpenWindowsAutomatically, WebkitLoadIconsIgnoringImageLoadSetting,
    WebkitMediaPlaybackAllowsInline, WebkitMediaPlaybackRequiresUserGesture, WebkitMinimumFontSize,
    WebkitMonospaceFontFamily, WebkitPictographFontFamily, WebkitPrintBackgrounds,
    WebkitSansSerifFontFamily, WebkitSerifFontFamily, WebkitUserAgent, WebkitZoomTextOnly,
};
use settings::{AppSettingsVariant, CookieAcceptPolicy};
use webkit2gtk::{CookieManagerExt, SettingsExt, WebContextExt, WebViewExt};

impl WebView {
    /// Set the cookie accept policy.
    fn set_cookie_accept(&self, cookie_accept: &CookieAcceptPolicy) {
        let cookie_manager = self
            .view
            .get_context()
            .and_then(|context| context.get_cookie_manager());
        if let Some(cookie_manager) = cookie_manager {
            cookie_manager.set_accept_policy(cookie_accept.to_webkit());
        }
    }

    /// Adjust the webkit settings.
    pub fn setting_changed(&self, setting: AppSettingsVariant) {
        if let Some(settings) = self.view.get_settings() {
            match setting {
                CookieAccept(ref value) => self.set_cookie_accept(value),
                HintChars(_) | HomePage(_) => (),
                WebkitAllowFileAccessFromFileUrls(value) => {
                    settings.set_allow_file_access_from_file_urls(value)
                }
                WebkitAllowModalDialogs(value) => settings.set_allow_modal_dialogs(value),
                WebkitAutoLoadImages(value) => settings.set_auto_load_images(value),
                WebkitCursiveFontFamily(ref value) => settings.set_cursive_font_family(value),
                WebkitDefaultCharset(ref value) => settings.set_default_charset(value),
                WebkitDefaultFontFamily(ref value) => settings.set_default_font_family(value),
                WebkitDefaultFontSize(value) => settings.set_default_font_size(value as u32),
                WebkitDefaultMonospaceFontSize(value) => {
                    settings.set_default_monospace_font_size(value as u32)
                }
                WebkitDrawCompositingIndicators(value) => {
                    settings.set_draw_compositing_indicators(value)
                }
                WebkitEnableAccelerated2dCanvas(value) => {
                    settings.set_enable_accelerated_2d_canvas(value)
                }
                WebkitEnableCaretBrowsing(value) => settings.set_enable_caret_browsing(value),
                WebkitEnableDeveloperExtras(value) => settings.set_enable_developer_extras(value),
                WebkitEnableDnsPrefetching(value) => settings.set_enable_dns_prefetching(value),
                WebkitEnableFrameFlattening(value) => settings.set_enable_frame_flattening(value),
                WebkitEnableFullscreen(value) => settings.set_enable_fullscreen(value),
                WebkitEnableHtml5Database(value) => settings.set_enable_html5_database(value),
                WebkitEnableHtml5LocalStorage(value) => {
                    settings.set_enable_html5_local_storage(value)
                }
                WebkitEnableHyperlinkAuditing(value) => {
                    settings.set_enable_hyperlink_auditing(value)
                }
                WebkitEnableJava(value) => settings.set_enable_java(value),
                WebkitEnableJavascript(value) => settings.set_enable_javascript(value),
                WebkitEnableMediaStream(value) => settings.set_enable_media_stream(value),
                WebkitEnableMediasource(value) => settings.set_enable_mediasource(value),
                WebkitEnableOfflineWebApplicationCache(value) => {
                    settings.set_enable_offline_web_application_cache(value)
                }
                WebkitEnablePageCache(value) => settings.set_enable_page_cache(value),
                WebkitEnablePlugins(value) => settings.set_enable_plugins(value),
                WebkitEnablePrivateBrowsing(value) => settings.set_enable_private_browsing(value),
                WebkitEnableResizableTextAreas(value) => {
                    settings.set_enable_resizable_text_areas(value)
                }
                WebkitEnableSiteSpecificQuirks(value) => {
                    settings.set_enable_site_specific_quirks(value)
                }
                WebkitEnableSmoothScrolling(value) => settings.set_enable_smooth_scrolling(value),
                WebkitEnableSpatialNavigation(value) => {
                    settings.set_enable_spatial_navigation(value)
                }
                WebkitEnableTabsToLinks(value) => settings.set_enable_tabs_to_links(value),
                WebkitEnableWebaudio(value) => settings.set_enable_webaudio(value),
                WebkitEnableWebgl(value) => settings.set_enable_webgl(value),
                WebkitEnableWriteConsoleMessagesToStdout(value) => {
                    settings.set_enable_write_console_messages_to_stdout(value)
                }
                WebkitEnableXssAuditor(value) => settings.set_enable_xss_auditor(value),
                WebkitFantasyFontFamily(ref value) => settings.set_fantasy_font_family(value),
                WebkitJavascriptCanAccessClipboard(value) => {
                    settings.set_javascript_can_access_clipboard(value)
                }
                WebkitJavascriptCanOpenWindowsAutomatically(value) => {
                    settings.set_javascript_can_open_windows_automatically(value)
                }
                WebkitLoadIconsIgnoringImageLoadSetting(value) => {
                    settings.set_load_icons_ignoring_image_load_setting(value)
                }
                WebkitMediaPlaybackAllowsInline(value) => {
                    settings.set_media_playback_allows_inline(value)
                }
                WebkitMediaPlaybackRequiresUserGesture(value) => {
                    settings.set_media_playback_requires_user_gesture(value)
                }
                WebkitMinimumFontSize(value) => settings.set_minimum_font_size(value as u32),
                WebkitMonospaceFontFamily(ref value) => settings.set_monospace_font_family(value),
                WebkitPictographFontFamily(ref value) => settings.set_pictograph_font_family(value),
                WebkitPrintBackgrounds(value) => settings.set_print_backgrounds(value),
                WebkitSansSerifFontFamily(ref value) => settings.set_sans_serif_font_family(value),
                WebkitSerifFontFamily(ref value) => settings.set_serif_font_family(value),
                WebkitUserAgent(ref value) => settings.set_user_agent(Some(value.as_str())),
                WebkitZoomTextOnly(value) => settings.set_zoom_text_only(value),
            }
        }
    }
}
