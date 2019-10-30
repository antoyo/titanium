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

#[derive(Commands)]
pub enum AppCommand {
    #[completion(hidden)]
    ActivateSelection,
    #[help(text="Update the host file used by the adblocker")]
    AdblockUpdate,
    #[help(text="Add a new user agent")]
    AddUserAgent(String),
    #[help(text="Go back in the history")]
    Back,
    #[special_command(incremental, identifier="?")]
    BackwardSearch(String),
    #[help(text="Add the current page to the bookmarks")]
    Bookmark,
    #[help(text="Delete the current page from the bookmarks")]
    BookmarkDel,
    #[help(text="Edit the bookmark tags of the current page")]
    BookmarkEditTags,
    #[help(text="Clear the browser cache")]
    ClearCache,
    #[help(text="Try to click link to next page if it exists")]
    ClickNextPage,
    #[help(text="Try to click link to the previous page if it exists")]
    ClickPrevPage,
    #[completion(hidden)]
    CopyLinkUrl,
    #[completion(hidden)]
    CopyUrl,
    #[help(text="Delete all the cookies")]
    DeleteAllCookies,
    #[help(text="Delete the cookies for the specified domain")]
    DeleteCookies(String),
    #[completion(hidden)]
    DeleteSelectedBookmark,
    #[completion(hidden)]
    FinishSearch,
    #[completion(hidden)]
    FocusInput,
    #[completion(hidden)]
    Follow,
    #[help(text="Go forward in the history")]
    Forward,
    #[completion(hidden)]
    GoMark(String),
    #[count]
    #[help(text="Go up one directory in url")]
    GoParentDir(Option<u32>),
    #[help(text="Go to root directory of url")]
    GoRootDir,
    #[completion(hidden)]
    HideHints,
    #[completion(hidden)]
    Hover,
    #[completion(hidden)]
    Insert,
    #[help(text="Open the web inspector")]
    Inspector,
    #[help(text="Kill the webview without confirmation")]
    KillWin,
    #[completion(hidden)]
    Mark(String),
    #[completion(hidden)]
    Normal,
    #[help(text="Open an URL")]
    Open(String),
    #[help(text="Delete the credentials for the current URL")]
    PasswordDelete,
    #[help(text="Insert a password in the focused text input")]
    PasswordInsert,
    #[help(text="Insert a password in the focused text input and submit the form")]
    PasswordInsertSubmit,
    #[help(text="Load the credentials in the login form")]
    PasswordLoad,
    #[help(text="Save the credentials from the login form")]
    PasswordSave,
    #[help(text="Load the credentials in the login form and submit the form")]
    PasswordSubmit,
    #[completion(hidden)]
    PasteUrl,
    #[help(text="Print the current page")]
    Print,
    #[help(text="Open an URL in a new private window")]
    PrivateWinOpen(String),
    #[help(text="Quit the application")]
    Quit,
    #[help(text="Reload the current page")]
    Reload,
    #[help(text="Reload the current page without using the cache")]
    ReloadBypassCache,
    #[help(text="Restore the opened pages after a crash")]
    RestoreUrls,
    #[completion(hidden)]
    SaveLink,
    #[completion(hidden)]
    SearchEngine(String),
    #[completion(hidden)]
    Screenshot(String),
    #[count]
    #[completion(hidden)]
    ScrollTo(Option<u32>),
    #[completion(hidden)]
    ScrollDown,
    #[completion(hidden)]
    ScrollDownHalf,
    #[completion(hidden)]
    ScrollDownLine,
    #[completion(hidden)]
    ScrollLeft,
    #[completion(hidden)]
    ScrollRight,
    #[completion(hidden)]
    ScrollTop,
    #[completion(hidden)]
    ScrollUp,
    #[completion(hidden)]
    ScrollUpHalf,
    #[completion(hidden)]
    ScrollUpLine,
    #[special_command(incremental, identifier="/")]
    Search(String),
    #[completion(hidden)]
    SearchNext,
    #[completion(hidden)]
    SearchPrevious,
    #[help(text="Select a user agent by name")]
    SelectUserAgent(String),
    #[help(text="Stop loading the current page")]
    Stop,
    #[completion(hidden)]
    UrlIncrement,
    #[completion(hidden)]
    UrlDecrement,
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
