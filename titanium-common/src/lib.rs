/*
 * Copyright (c) 2016-2017 Boucher, Antoni <bouanto@zoho.com>
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

//! Message used to communicate between the UI and the web processes.

#![warn(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
)]

#[macro_use]
extern crate serde_derive;

// TODO: put in the home directory.
/// The path to the unix domain socket.
pub const PATH: &[u8] = b"titanium-server";

#[doc(hidden)]
pub type ExtensionId = u64;
#[doc(hidden)]
pub type PageId = u64;

/// Action that should be executed from the UI process.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Action {
    /// Copy the specified link in the clipboard.
    CopyLink(String),
    /// Download the specified destination.
    DownloadLink(String),
    /// Show the file input.
    FileInput,
    /// Go in insert mode.
    GoInInsertMode,
    /// No action.
    NoAction,
}

/// The mode for the follow mode.
/// This indicates the action that will be taken after an hint is selected.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum FollowMode {
    /// The link will be clicked.
    Click,
    /// The URL of the link will be copied.
    CopyLink,
    /// The source of the link will be downloaded.
    Download,
    /// The cursor will move over the link.
    Hover,
}

/// Message with the associated window/page id.
#[derive(Debug, Deserialize, Serialize)]
pub struct Message(pub PageId, pub InnerMessage);

/// Message representing actions to to in the web page.
// Switch to SimpleMsg to avoid these empty ().
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum InnerMessage {
    /// Response to ActivateHint.
    ActivateAction(Action),
    /// Activate the selected hint according to the specified follow mode.
    ActivateHint(FollowMode, bool),
    /// Click on the link in the selection.
    ActivateSelection(),
    /// Response to EnterHintKey.
    ClickHintElement(),
    /// Regex lookup next page link to click
    ClickNextPage(),
    /// Regex lookup prev page link to click
    ClickPrevPage(),
    /// Response to GetCredentials.
    Credentials(String, String),
    /// Add a key to the current hint text.
    EnterHintKey(char),
    /// Response to FocusInput.
    EnterInsertMode(),
    /// Focus the first text input.
    FocusInput(),
    /// Ask for the credentials from the login form.
    GetCredentials(),
    /// Send the page ID to the application to connect the web extension with the right window.
    /// Answer to GetId.
    Id(ExtensionId, PageId),
    /// Insert some text in the currently focused text field.
    InsertText(String),
    /// Hide the hints.
    HideHints(),
    /// Write the username and password in the login form.
    LoadUsernamePass(String, String),
    /// Open the given URL.
    /// This is used when starting a new titanium process to tell the existing process to open a
    /// new window.
    Open(Vec<String>),
    /// Set the scrolling element.
    ResetScrollElement(),
    /// Scroll vertically by the specified amount of pixels.
    ScrollBy(i64),
    /// Scroll horizontally by the specified amount of pixels.
    ScrollByX(i64),
    /// Send the scroll percentage to the app.
    ScrollPercentage(Percentage),
    /// Scroll to the top of the web page.
    ScrollTop(),
    /// Scroll to the speficied percentage of the web page
    ScrollToPercent(u32),
    /// Set the selected file on a file input.
    SelectFile(String),
    /// Show the hints over the elements.
    ShowHints(String),
    /// Submit the login form.
    SubmitLoginForm(),
}

/// Either all the page is shown (hence, no percentage) or a value between 0 and 100.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Percentage {
    /// No percentage, since all the page is shown.
    All,
    /// A scroll percentage between 0 and 100.
    Percent(i64),
}
