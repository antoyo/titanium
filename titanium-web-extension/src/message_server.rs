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

#![allow(non_upper_case_globals)]

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use glib::Cast;
use titanium_common::Action::{FileInput, GoInInsertMode, NoAction};
use webkit2gtk_webextension::{
    DOMDOMWindowExtManual,
    DOMDocumentExt,
    DOMElement,
    DOMElementExt,
    DOMHTMLElement,
    DOMHTMLElementExt,
    DOMHTMLInputElement,
    DOMHTMLSelectElement,
    DOMHTMLTextAreaElement,
    DOMNodeExt,
    WebExtension,
};

use dom::{
    ElementIter,
    get_body,
    is_enabled,
    is_hidden,
    is_text_input,
    mouse_down,
    mouse_out,
    mouse_over,
};
use hints::{create_hints, hide_unrelevant_hints, show_all_hints, HINTS_ID};
use login_form::{get_credentials, load_password, load_username, submit_login_form};
use scroll::Scrollable;

macro_rules! get_page {
    ($_self:ident) => {
        $_self.extension.get_page($_self.page_id.get())
    };
}

macro_rules! get_document {
    ($_self:ident) => {{
        let document = get_page!($_self)
            .and_then(|page| page.get_dom_document());
        if let Some(document) = document {
            document
        }
        else {
            return;
        }
    }};
}

dbus_class!("com.titanium.client", class MessageServer
    ( page_id: Rc<Cell<u64>>
    , extension: WebExtension
    , hint_keys: String
    , hint_map: HashMap<String, DOMElement>
    , activated_file_input: Option<DOMHTMLInputElement>
    , last_hovered_element: Option<DOMElement>
    )
{
    // Activate (click, focus, hover) the selected hint.
    // Return an Action that should be executed from the UI process.
    fn activate_hint(&mut self, follow_mode: &str) -> i32 {
        fn click(server: &mut DBusObject, element: DOMHTMLElement) -> i32 {
            if element.is::<DOMHTMLInputElement>() {
                if let Ok(input_element) = element.clone().downcast::<DOMHTMLInputElement>() {
                    let input_type = input_element.get_input_type();
                    if let Some(input_type) = input_type {
                        match input_type.as_ref() {
                            "button" | "checkbox" | "image" | "radio" | "reset" | "submit" => element.click(),
                            // FIXME: file and color not opening.
                            "color" => (),
                            "file" => {
                                server.activated_file_input = Some(input_element);
                                return FileInput as i32
                            },
                            _ => {
                                element.focus();
                                return GoInInsertMode as i32;
                            },
                        }
                    }
                }
            }
            else if element.is::<DOMHTMLTextAreaElement>() {
                element.focus();
                return GoInInsertMode as i32;
            }
            else if element.is::<DOMHTMLSelectElement>() {
                if element.get_attribute("multiple").is_some() {
                    element.focus();
                    return GoInInsertMode as i32;
                }
                else {
                    mouse_down(&element.upcast());
                }
            }
            else {
                element.click();
            }
            NoAction as i32
        }

        fn hover(server: &mut DBusObject, element: DOMHTMLElement) -> i32 {
            if let Some(ref element) = server.last_hovered_element {
                mouse_out(element);
            }
            server.last_hovered_element = Some(element.clone().upcast());
            mouse_over(&element.upcast());
            NoAction as i32
        }

        let follow =
            if follow_mode == "hover" {
                hover
            }
            else {
                click
            };

        let element = self.hint_map.get(&self.hint_keys)
            .and_then(|element| element.clone().downcast::<DOMHTMLElement>().ok());
        if let Some(element) = element {
            self.hide_hints();
            self.hint_map.clear();
            self.hint_keys.clear();
            return follow(self, element);
        }
        NoAction as i32
    }

    // Click on the link of the selected text.
    fn activate_selection(&self) -> () {
        let result = get_page!(self)
            .and_then(|page| page.get_dom_document())
            .and_then(|document| document.get_default_view())
            .and_then(|window| window.get_selection())
            .and_then(|selection| selection.get_anchor_node())
            .and_then(|anchor_node| anchor_node.get_parent_element())
            .and_then(|parent| parent.downcast::<DOMHTMLElement>().ok());
        if let Some(parent) = result {
            parent.click();
        }
    }

    // Handle the key press event for the hint mode.
    // This hides the hints that are not relevant anymore.
    // Return true if an element should be clicked.
    fn enter_hint_key(&mut self, key: char) -> bool {
        self.hint_keys.push(key);
        let element = self.hint_map.get(&self.hint_keys)
            .and_then(|element| element.clone().downcast::<DOMHTMLElement>().ok());
        // If no element is found, hide the unrelevant hints.
        let result = element.is_some();
        if !result {
            let document = get_page!(self)
                .and_then(|page| page.get_dom_document());
            if let Some(document) = document {
                let all_hidden = hide_unrelevant_hints(&document, &self.hint_keys);
                if all_hidden {
                    self.hint_keys.clear();
                    show_all_hints(&document);
                }
            }
        }
        result
    }

    // Focus the first input element.
    // Returns true if an element was focused.
    fn focus_input(&self) -> bool {
        let document = get_page!(self)
            .and_then(|page| page.get_dom_document());
        if let Some(document) = document {
            let tag_names = ["input", "textarea"];
            for tag_name in &tag_names {
                let iter = ElementIter::new(document.get_elements_by_tag_name(tag_name));
                for element in iter {
                    if !is_hidden(&document, &element) && is_enabled(&element) && is_text_input(&element) {
                        element.focus();
                        return true;
                    }
                }
            }
        }
        false
    }

    // Get the username and password from the login form.
    fn get_credentials(&self) -> (String, String) {
        let mut username = String::new();
        let mut password = String::new();
        let credential = get_page!(self)
            .and_then(|page| page.get_dom_document())
            .and_then(|document| get_credentials(&document));
        if let Some(credential) = credential {
            username = credential.username;
            password = credential.password;
        }
        (username, password)
    }

    // Get the page scroll percentage.
    fn get_scroll_percentage(&self) -> i64 {
        if let Some(page) = get_page!(self) {
            page.scroll_percentage()
        }
        else {
            0
        }
    }

    // Hide all the hints.
    fn hide_hints(&self) -> () {
        let page = get_page!(self);
        let elements = page.as_ref()
            .and_then(|page| page.get_dom_document())
            .and_then(|document| document.get_element_by_id(HINTS_ID))
            .and_then(|hints| page.as_ref().and_then(|page| get_body(page).map(|body| (hints, body))));
        if let Some((hints, body)) = elements {
            body.remove_child(&hints).ok();
        }
    }

    // FIXME: use one method with two parameters to load the username and the password at the same
    // time.
    // Load the password in the login form.
    fn load_password(&self, password: &str) -> () {
        let document = get_document!(self);
        load_password(&document, password);
    }

    // Load the username in the login form.
    fn load_username(&self, username: &str) -> () {
        let document = get_document!(self);
        load_username(&document, username);
    }

    // Scroll to the bottom of the page.
    fn scroll_bottom(&self) -> () {
        if let Some(page) = get_page!(self) {
            page.scroll_bottom();
        }
    }

    // Scroll by the specified amount of pixels.
    fn scroll_by(&self, pixels: i64) -> () {
        if let Some(page) = get_page!(self) {
            page.scroll_by(pixels);
        }
    }

    // Scroll horizontally by the specified amount of pixels.
    fn scroll_by_x(&self, pixels: i64) -> () {
        if let Some(page) = get_page!(self) {
            page.scroll_by_x(pixels);
        }
    }

    // Scroll to the top of the page.
    fn scroll_top(&self) -> () {
        if let Some(page) = get_page!(self) {
            page.scroll_top();
        }
    }

    // Set the selected file on the input[type="file"].
    fn select_file(&mut self, file: &str) -> () {
        if let Some(ref input_file) = self.activated_file_input {
            // FIXME: this is not working.
            input_file.set_value(file);
        }
        self.activated_file_input = None;
    }

    // Show the hint of elements using the hint characters.
    fn show_hints(&mut self, hint_chars: &str) -> () {
        self.hint_keys.clear();
        let page = get_page!(self);
        let body = page.as_ref().and_then(|page| get_body(page));
        let document = page.and_then(|page| page.get_dom_document());
        if let (Some(document), Some(body)) = (document, body) {
            if let Some((hints, hint_map)) = create_hints(&document, hint_chars) {
                self.hint_map = hint_map;
                body.append_child(&hints).ok();
            }
        }
    }

    // Submit the login form.
    fn submit_login_form(&self) -> () {
        let document = get_document!(self);
        submit_login_form(&document);
    }
});
