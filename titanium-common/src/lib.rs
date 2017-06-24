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
    unused_results,
)]

#[macro_use]
extern crate serde_derive;

/// Action that should be executed from the UI process.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Action {
    FileInput,
    GoInInsertMode,
    NoAction,
}

// Switch to SimpleMsg to avoid these empty ().
#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    /// Response to ActivateHint.
    ActivateAction(Action),
    ActivateHint(String),
    ActivateSelection(),
    /// Response to EnterHintKey.
    ClickHintElement(),
    /// Response to GetCredentials.
    Credentials(String, String),
    EnterHintKey(char),
    /// Response to FocusInput.
    EnterInsertMode(),
    FocusInput(),
    GetCredentials(),
    GetScrollPercentage(),
    HideHints(),
    LoadUsernamePass(String, String),
    ScrollBottom(),
    ScrollBy(i64),
    ScrollByX(i64),
    /// Response of GetScrollPercentage.
    ScrollPercentage(Percentage),
    ScrollTop(),
    SelectFile(String),
    ShowHints(String),
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
