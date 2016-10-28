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

#[derive(Commands)]
pub enum AppCommand {
    #[completion(hidden)]
    ActivateSelection,
    #[help(text="Go back in the history")]
    Back,
    #[completion(hidden)]
    CopyUrl,
    #[completion(hidden)]
    FinishSearch,
    #[completion(hidden)]
    Follow,
    #[help(text="Go forward in the history")]
    Forward,
    #[completion(hidden)]
    HideHints,
    #[completion(hidden)]
    Insert,
    #[help(text="Open the web inspector")]
    Inspector,
    #[completion(hidden)]
    Normal,
    #[help(text="Open an URL")]
    Open(String),
    #[completion(hidden)]
    PasteUrl,
    #[help(text="Quit the application")]
    Quit,
    #[help(text="Reload the current page")]
    Reload,
    #[help(text="Reload the current page without using the cache")]
    ReloadBypassCache,
    #[completion(hidden)]
    SearchEngine(String),
    #[completion(hidden)]
    ScrollBottom,
    #[completion(hidden)]
    ScrollDown,
    #[completion(hidden)]
    ScrollDownHalf,
    #[completion(hidden)]
    ScrollDownLine,
    #[completion(hidden)]
    ScrollTop,
    #[completion(hidden)]
    ScrollUp,
    #[completion(hidden)]
    ScrollUpHalf,
    #[completion(hidden)]
    ScrollUpLine,
    #[completion(hidden)]
    SearchNext,
    #[completion(hidden)]
    SearchPrevious,
    #[help(text="Stop loading the current page")]
    Stop,
    #[completion(hidden)]
    WinFollow,
    #[help(text="Open an URL in a new window")]
    WinOpen(String),
    #[completion(hidden)]
    WinPasteUrl,
    #[help(text="Zoom the current page in")]
    ZoomIn,
    #[help(text="Zoom the current page to 100%")]
    ZoomNormal,
    #[help(text="Zoom the current page out")]
    ZoomOut,
}

special_commands!(SpecialCommand {
    BackwardSearch('?', always),
    Search('/', always),
});
