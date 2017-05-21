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

//! Message server interface.

/*dbus_interface!(
#[dbus("com.titanium.client")]
interface MessageServer {
    fn activate_hint(&mut self, follow_mode: &str) -> i32;
    fn activate_selection(&self);
    fn enter_hint_key(&mut self, key: char) -> bool;
    fn focus_input(&self) -> bool;
    fn get_credentials(&self) -> (String, String);
    fn get_scroll_percentage(&self) -> i64;
    fn hide_hints(&self);
    fn load_password(&self, password: &str);
    fn load_username(&self, username: &str);
    fn scroll_bottom(&self);
    fn scroll_by(&self, pixels: i64);
    fn scroll_by_x(&self, pixels: i64);
    fn scroll_top(&self);
    fn select_file(&self, file: &str);
    fn show_hints(&mut self, hint_chars: &str);
    fn submit_login_form(&self);
}
);*/
