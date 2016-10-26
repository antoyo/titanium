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

use std::collections::HashMap;

use glib::object::Downcast;
use webkit2gtk_webextension::{
    DOMDocument,
    DOMDocumentExt,
    DOMElement,
    DOMElementExt,
    DOMHTMLIFrameElement,
    DOMNodeExt,
};

use dom::{Pos, get_position, hide, is_enabled, is_visible, show};

pub const HINTS_ID: &'static str = "__titanium_hints";

pub struct Hints {
    characters: String,
    current_indexes: Vec<usize>,
    hints: HashMap<String, DOMElement>,
    size: usize,
}

impl Hints {
    fn new(size: usize, hint_chars: &str) -> Self {
        let characters = hint_chars.to_string();
        let first_index =
            if size <= characters.len() {
                0
            }
            else {
                characters.len() / 2
            };
        Hints {
            characters: characters,
            current_indexes: vec![first_index],
            hints: HashMap::new(),
            size: size,
        }
    }

    fn add(&mut self, element: &DOMElement) -> String {
        let mut hint = String::new();
        if self.size <= self.characters.len() {
            let index = self.current_indexes[0];
            hint.push(self.characters.chars().nth(index).unwrap_or('a'));
            self.current_indexes[0] += 1;
        }
        else {
            for &index in &self.current_indexes {
                hint.push(self.characters.chars().nth(index).unwrap_or('a'));
            }
            let mut changed = true;
            for current_index in self.current_indexes.iter_mut().rev() {
                if !changed {
                    break;
                }
                changed = false;
                let mut index = *current_index;
                index += 1;
                if index >= self.characters.len() {
                    changed = true;
                    index = 0;
                }
                *current_index = index;
            }
            if changed {
                self.current_indexes.push(self.characters.len() / 2);
            }
        }
        self.hints.insert(hint.clone(), element.clone());
        hint
    }
}

fn create_hint(document: &DOMDocument, pos: Pos, hint_text: &str) -> Option<DOMElement> {
    document.create_element("div").ok().map(|hint| {
        hint.set_class_name("__titanium_hint");
        hint.set_id(&format!("__titanium_hint_{}", hint_text));
        if let Some(style) = hint.get_style() {
            style.set_property("position", "absolute", "").ok();
            style.set_property("left", &format!("{}px", pos.x), "").ok();
            style.set_property("top", &format!("{}px", pos.y), "").ok();
            style.set_property("z-index", "10000", "").ok();
        }
        if let Some(text) = document.create_text_node(hint_text) {
            hint.append_child(&text).ok();
        }
        hint
    })
}

/// Create the hints over all the elements that can be activated by the user (links, form elements).
pub fn create_hints(document: &DOMDocument, hint_chars: &str) -> Option<(DOMElement, HashMap<String, DOMElement>)> {
    document.create_element("div").ok().map(|hints| {
        hints.set_id(HINTS_ID);
        if let Some(style) = hints.get_style() {
            style.set_property("position", "absolute", "").ok();
            style.set_property("left", "0", "").ok();
            style.set_property("top", "0", "").ok();
        }

        let elements_to_hint = get_elements_to_hint(document);

        let mut hint_map = Hints::new(elements_to_hint.len(), hint_chars);
        for element in elements_to_hint {
            if let Some(pos) = get_position(&element) {
                if let Some(hint) = create_hint(&document, pos, &hint_map.add(&element)) {
                    hints.append_child(&hint).ok();
                }
            }
        }
        (hints, hint_map.hints)
    })
}

/// Get the elements to hint.
fn get_elements_to_hint(document: &DOMDocument) -> Vec<DOMElement> {
    let mut elements_to_hint = get_hintable_elements(document);
    elements_to_hint.append(&mut get_input_elements(document));
    elements_to_hint.append(&mut get_hintable_elements_from_iframes(document));
    elements_to_hint
}

/// Get the hintable elements, except the input.
fn get_hintable_elements(document: &DOMDocument) -> Vec<DOMElement> {
    let mut elements_to_hint = vec![];
    let tag_names = ["a", "button", "select", "textarea"];
    for tag_name in &tag_names {
        let elements = document.get_elements_by_tag_name(tag_name);
        if let Some(elements) = elements {
            for i in 0 .. elements.get_length() {
                if let Some(element) = elements.item(i) {
                    if let Ok(element) = element.downcast() {
                        // Only show the hints for visible elements that are not disabled.
                        if is_visible(&document, &element) && is_enabled(&element) {
                            elements_to_hint.push(element);
                        }
                    }
                }
            }
        }
    }
    elements_to_hint
}

/// Get the hintable elements from the iframes.
fn get_hintable_elements_from_iframes(document: &DOMDocument) -> Vec<DOMElement> {
    let mut elements_to_hint = vec![];
    if let Some(iframes) = document.get_elements_by_tag_name("iframe") {
        for i in 0 .. iframes.get_length() {
            if let Some(iframe) = iframes.item(i) {
                if let Ok(iframe) = iframe.downcast() {
                    let iframe: DOMHTMLIFrameElement = iframe;
                    if let Some(iframe_document) = iframe.get_content_document() {
                        elements_to_hint.append(&mut get_elements_to_hint(&iframe_document));
                    }
                }
            }
        }
    }
    elements_to_hint
}

/// Get the hintable input elements.
fn get_input_elements(document: &DOMDocument) -> Vec<DOMElement> {
    let mut elements_to_hint = vec![];
    let form_elements = document.get_elements_by_tag_name("input");
    if let Some(form_elements) = form_elements {
        for i in 0 .. form_elements.get_length() {
            if let Some(element) = form_elements.item(i) {
                if let Ok(element) = element.downcast() {
                    if is_visible(&document, &element) && is_enabled(&element) {
                        // Do not show hints for hidden form elements.
                        if element.get_attribute("type") != Some("hidden".to_string()) {
                            elements_to_hint.push(element);
                        }
                    }
                }
            }
        }
    }
    elements_to_hint
}

/// Hide the hints that does not start with `hint_keys`.
pub fn hide_unrelevant_hints(document: &DOMDocument, hint_keys: &str) -> bool {
    let all_hints = document.query_selector_all(&format!(".__titanium_hint"));
    let hints_to_hide = document.query_selector_all(&format!(".__titanium_hint:not([id^=\"__titanium_hint_{}\"])", hint_keys));
    if let Ok(hints) = hints_to_hide {
        for i in 0 .. hints.get_length() {
            let hint = hints.item(i).and_then(|hint| hint.downcast().ok());
            if let Some(hint_element) = hint {
                hide(&hint_element);
            }
        }
        if let Ok(all_hints) = all_hints {
            return all_hints.get_length() == hints.get_length()
        }
    }
    false
}

/// Show all hints.
pub fn show_all_hints(document: &DOMDocument) {
    let all_hints = document.query_selector_all(&format!(".__titanium_hint"));
    if let Ok(hints) = all_hints {
        for i in 0 .. hints.get_length() {
            let hint = hints.item(i).and_then(|hint| hint.downcast().ok());
            if let Some(hint_element) = hint {
                show(&hint_element);
            }
        }
    }
}
