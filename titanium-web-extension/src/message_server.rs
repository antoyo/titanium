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
use webkit2gtk_webextension::{DOMDOMWindowExtManual, DOMDocumentExt, DOMElement, DOMElementExt, DOMHTMLElement, DOMHTMLElementExt, DOMHTMLInputElement, DOMHTMLSelectElement, DOMHTMLTextAreaElement, DOMNodeExt, WebExtension};

use dom::{get_body, mouse_down};
use hints::{create_hints, hide_unrelevant_hints, HINTS_ID};
use scroll::Scrollable;

macro_rules! get_page {
    ($_self:ident) => {
        $_self.extension.get_page($_self.page_id.get())
    };
}

dbus_class!("com.titanium.client", class MessageServer
    ( page_id: Rc<Cell<u64>>
    , extension: WebExtension
    , hint_keys: String
    , hint_map: HashMap<String, DOMElement>
    )
{
    // Return true if a text element has been focused.
    fn activate_hint(&mut self) -> bool {
        let element = self.hint_map.get(&self.hint_keys)
            .and_then(|element| element.clone().downcast::<DOMHTMLElement>().ok());
        if let Some(element) = element {
            self.hide_hints();
            self.hint_map.clear();
            self.hint_keys.clear();
            let input_element: Result<DOMHTMLInputElement, _> = element.clone().downcast();
            let select_element: Result<DOMHTMLSelectElement, _> = element.clone().downcast();
            let textarea_element: Result<DOMHTMLTextAreaElement, _> = element.clone().downcast();
            if let Ok(input_element) = input_element {
                if let Some(input_type) = input_element.get_input_type() {
                    match input_type.as_ref() {
                        "button" | "checkbox" | "image" | "radio" | "reset" | "submit" => input_element.click(),
                        // FIXME: file and color not opening.
                        "color" | "file" => {
                            mouse_down(input_element.upcast());
                        },
                        _ => {
                            input_element.focus();
                            return true;
                        },
                    }
                }
            }
            else if let Ok(textarea_element) = textarea_element {
                textarea_element.focus();
                return true;
            }
            else if let Ok(select_element) = select_element {
                mouse_down(select_element.upcast());
            }
            else {
                element.click();
            }
        }
        false
    }

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
                hide_unrelevant_hints(&document, &self.hint_keys);
            }
        }
        result
    }

    fn get_scroll_percentage(&self) -> i64 {
        if let Some(page) = get_page!(self) {
            page.scroll_percentage()
        }
        else {
            0
        }
    }

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

    fn scroll_bottom(&self) -> () {
        if let Some(page) = get_page!(self) {
            page.scroll_bottom();
        }
    }

    fn scroll_by(&self, pixels: i64) -> () {
        if let Some(page) = get_page!(self) {
            page.scroll_by(pixels);
        }
    }

    fn scroll_top(&self) -> () {
        if let Some(page) = get_page!(self) {
            page.scroll_top();
        }
    }

    fn show_hint_on_links(&mut self) -> () {
        self.hint_keys.clear();
        let page = get_page!(self);
        let body = page.as_ref().and_then(|page| get_body(page));
        let document = page.and_then(|page| page.get_dom_document());
        if let (Some(document), Some(body)) = (document, body) {
            if let Some((hints, hint_map)) = create_hints(&document) {
                self.hint_map = hint_map;
                body.append_child(&hints).ok();
            }
        }
    }
});
