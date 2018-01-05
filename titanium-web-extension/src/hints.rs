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

use std::collections::HashMap;
use std::i32;

use glib::object::Downcast;
use webkit2gtk_webextension::{
    DOMCSSStyleDeclarationExt,
    DOMDocument,
    DOMDocumentExt,
    DOMElement,
    DOMElementExt,
    DOMHTMLIFrameElement,
    DOMHTMLIFrameElementExt,
    DOMNodeExt,
    DOMNodeListExt,
};

use dom::{
    NodeIter,
    Pos,
    get_position,
    hide,
    is_enabled,
    is_visible,
    show,
};

pub const HINTS_ID: &'static str = "__titanium_hints";

pub struct Hints {
    hints: HashMap<String, DOMElement>,
    prefix_indexes: Vec<usize>,
    prefixes: String,
    suffix_index: usize,
    suffixes: String,
}

impl Hints {
    fn new(count: usize, hint_chars: &str) -> Self {
        let (prefixes, suffixes) =
            if count <= hint_chars.len() {
                (String::new(), hint_chars.to_string())
            }
            else {
                let (prefixes, suffixes) = hint_chars.split_at(hint_chars.len() / 2);
                (prefixes.to_string(), suffixes.to_string())
            };
        Hints {
            hints: HashMap::new(),
            prefix_indexes: vec![],
            prefixes: prefixes,
            suffix_index: 0,
            suffixes: suffixes,
        }
    }

    /// Add an hint for the specified element.
    /// Returns the text of that hint.
    fn add(&mut self, element: &DOMElement) -> String {
        let hint = self.generate();
        self.hints.insert(hint.clone(), element.clone());
        hint
    }

    /// Generate the next hint text.
    fn generate(&mut self) -> String {
        let suffix = self.suffixes.chars().nth(self.suffix_index).unwrap_or('a');
        self.suffix_index += 1;

        let prefix: String = self.prefix_indexes
            .iter()
            .map(|&index| self.prefixes.chars().nth(index).unwrap_or('a'))
            .collect();

        if self.suffix_index >= self.suffixes.len() {
            self.suffix_index = 0;
            let mut i = 0;
            while i < self.prefix_indexes.len() {
                self.prefix_indexes[i] += 1;
                if self.prefix_indexes[i] >= self.prefixes.len() {
                    self.prefix_indexes[i] = 0;
                }
                else {
                    break;
                }
                i += 1;
            }
            if i >= self.prefix_indexes.len() {
                self.prefix_indexes.push(0);
            }
        }

        format!("{}{}", prefix, suffix)
    }
}

fn create_hint(document: &DOMDocument, pos: &Pos, hint_text: &str) -> Option<DOMElement> {
    document.create_element("div").ok().and_then(|hint| {
        hint.set_class_name("__titanium_hint");
        hint.set_id(&format!("__titanium_hint_{}", hint_text));
        let style = wtry_opt!(hint.get_style());
        check_err_opt!(style.set_property("position", "absolute", "").ok());
        check_err_opt!(style.set_property("left", &format!("{}px", pos.x), "").ok());
        check_err_opt!(style.set_property("top", &format!("{}px", pos.y), "").ok());
        check_err_opt!(style.set_property("z-index", &i32::MAX.to_string(), "").ok());

        let text = wtry_opt!(document.create_text_node(hint_text));
        check_err_opt!(hint.append_child(&text).ok());
        Some(hint)
    })
}

/// Create the hints over all the elements that can be activated by the user (links, form elements).
pub fn create_hints(document: &DOMDocument, hint_chars: &str) -> Option<(DOMElement, HashMap<String, DOMElement>)> {
    document.create_element("div").ok().and_then(|hints| {
        hints.set_id(HINTS_ID);
        let style = wtry_opt!(hints.get_style());
        check_err_opt!(style.set_property("position", "absolute", "").ok());
        check_err_opt!(style.set_property("left", "0", "").ok());
        check_err_opt!(style.set_property("top", "0", "").ok());

        let elements_to_hint = get_elements_to_hint(document);

        let mut hint_map = Hints::new(elements_to_hint.len(), hint_chars);
        for element in elements_to_hint {
            if let Some(mut pos) = get_position(&element) {
                // FIXME: adjust the position to avoid showing the hint outside the viewport.
                let hint = wtry_opt!(create_hint(document, &pos, &hint_map.add(&element)));
                check_err_opt!(hints.append_child(&hint).ok());
            }
        }
        Some((hints, hint_map.hints))
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
        let elements = NodeIter::new(document.get_elements_by_tag_name(tag_name));
        for element in elements {
            // Only show the hints for visible elements that are not disabled.
            // TODO: might not need to check if the element is visible anymore because in this
            // case, get_position() returns None.
            if is_visible(document, &element) && is_enabled(&element) {
                elements_to_hint.push(element);
            }
        }
    }
    elements_to_hint
}

/// Get the hintable elements from the iframes.
fn get_hintable_elements_from_iframes(document: &DOMDocument) -> Vec<DOMElement> {
    let mut elements_to_hint = vec![];
    let iter = NodeIter::new(document.get_elements_by_tag_name("iframe"));
    for iframe in iter {
        if let Ok(iframe) = iframe.downcast() {
            let iframe: DOMHTMLIFrameElement = iframe;
            if let Some(iframe_document) = iframe.get_content_document() {
                elements_to_hint.append(&mut get_elements_to_hint(&iframe_document));
            }
        }
    }
    elements_to_hint
}

/// Get the hintable input elements.
fn get_input_elements(document: &DOMDocument) -> Vec<DOMElement> {
    let mut elements_to_hint = vec![];
    let form_elements = NodeIter::new(document.get_elements_by_tag_name("input"));
    for element in form_elements {
        if is_visible(document, &element) && is_enabled(&element) &&
            // Do not show hints for hidden form elements.
            element.get_attribute("type") != Some("hidden".to_string())
        {
            elements_to_hint.push(element);
        }
    }
    elements_to_hint
}

/// Hide the hints that does not start with `hint_keys`.
pub fn hide_unrelevant_hints(document: &DOMDocument, hint_keys: &str) -> bool {
    let hints_to_hide = document.query_selector_all(
        &format!(".__titanium_hint:not([id^=\"__titanium_hint_{}\"])", hint_keys))
        .ok();
    let hints_len = hints_to_hide.as_ref().map(|hints| hints.get_length()).unwrap_or(0);
    let hints = NodeIter::new(hints_to_hide);
    for hint in hints {
        hide(&hint);
    }
    let all_hints = unwrap_or_ret!(document.query_selector_all(".__titanium_hint"), false);
    all_hints.get_length() == hints_len
}

/// Show all hints.
pub fn show_all_hints(document: &DOMDocument) {
    let hints = NodeIter::new(document.query_selector_all(".__titanium_hint").ok());
    for hint in hints {
        show(&hint);
    }
}

#[cfg(test)]
mod tests {
    use super::Hints;

    #[test]
    fn generate_hints() {
        let expected_hints = vec![
            "h", "j", "k", "l", "a", "s", "d", "f", "g", "y", "u", "i", "o",
        ];
        let count = expected_hints.len();
        let mut hints = Hints::new(count, "hjklasdfgyuiopqwertnmzxcvb");
        let mut hint_texts = vec![];
        for _ in 0 .. count {
            hint_texts.push(hints.generate());
        }
        assert_eq!(expected_hints, hint_texts);

        let expected_hints = vec![
            "h", "j", "k", "l", "a", "s", "d", "f", "g", "y", "u", "i", "o", "p", "q", "w", "e", "r", "t", "n", "m",
            "z", "x", "c", "v", "b",
        ];
        let count = expected_hints.len();
        let mut hints = Hints::new(count, "hjklasdfgyuiopqwertnmzxcvb");
        let mut hint_texts = vec![];
        for _ in 0 .. count {
            hint_texts.push(hints.generate());
        }
        assert_eq!(expected_hints, hint_texts);

        let expected_hints = vec![
            "p", "q", "w", "e", "r", "t", "n", "m", "z", "x", "c", "v", "b",
            "hp", "hq", "hw", "he", "hr", "ht", "hn", "hm", "hz", "hx", "hc", "hv", "hb",
            "jp", "jq", "jw", "je", "jr", "jt", "jn", "jm", "jz", "jx", "jc", "jv", "jb",
            "kp", "kq", "kw", "ke", "kr", "kt", "kn", "km", "kz", "kx", "kc", "kv", "kb",
            "lp", "lq", "lw", "le", "lr", "lt", "ln", "lm", "lz", "lx", "lc", "lv", "lb",
            "ap", "aq", "aw", "ae", "ar", "at", "an", "am", "az", "ax", "ac", "av", "ab",
            "sp", "sq", "sw", "se", "sr", "st", "sn", "sm", "sz", "sx", "sc", "sv", "sb",
            "dp", "dq", "dw", "de", "dr", "dt", "dn", "dm", "dz", "dx", "dc", "dv", "db",
            "fp", "fq", "fw", "fe", "fr", "ft", "fn", "fm", "fz", "fx", "fc", "fv", "fb",
            "gp", "gq", "gw", "ge", "gr", "gt", "gn", "gm", "gz", "gx", "gc", "gv", "gb",
            "yp", "yq", "yw", "ye", "yr", "yt", "yn", "ym", "yz", "yx", "yc", "yv", "yb",
            "up", "uq", "uw", "ue", "ur", "ut", "un", "um", "uz", "ux", "uc", "uv", "ub",
            "ip", "iq", "iw", "ie", "ir", "it", "in", "im", "iz", "ix", "ic", "iv", "ib",
            "op", "oq", "ow", "oe", "or", "ot", "on", "om", "oz", "ox", "oc", "ov", "ob",
        ];
        let count = expected_hints.len();
        let mut hints = Hints::new(count, "hjklasdfgyuiopqwertnmzxcvb");
        let mut hint_texts = vec![];
        for _ in 0 .. count {
            hint_texts.push(hints.generate());
        }
        assert_eq!(expected_hints, hint_texts);

        let expected_hints = vec![
            "p", "q", "w", "e", "r", "t", "n", "m", "z", "x", "c", "v", "b",
            "hp", "hq", "hw", "he", "hr", "ht", "hn", "hm", "hz", "hx", "hc", "hv", "hb",
            "jp", "jq", "jw", "je", "jr", "jt", "jn", "jm", "jz", "jx", "jc", "jv", "jb",
            "kp", "kq", "kw", "ke", "kr", "kt", "kn", "km", "kz", "kx", "kc", "kv", "kb",
            "lp", "lq", "lw", "le", "lr", "lt", "ln", "lm", "lz", "lx", "lc", "lv", "lb",
            "ap", "aq", "aw", "ae", "ar", "at", "an", "am", "az", "ax", "ac", "av", "ab",
            "sp", "sq", "sw", "se", "sr", "st", "sn", "sm", "sz", "sx", "sc", "sv", "sb",
            "dp", "dq", "dw", "de", "dr", "dt", "dn", "dm", "dz", "dx", "dc", "dv", "db",
            "fp", "fq", "fw", "fe", "fr", "ft", "fn", "fm", "fz", "fx", "fc", "fv", "fb",
            "gp", "gq", "gw", "ge", "gr", "gt", "gn", "gm", "gz", "gx", "gc", "gv", "gb",
            "yp", "yq", "yw", "ye", "yr", "yt", "yn", "ym", "yz", "yx", "yc", "yv", "yb",
            "up", "uq", "uw", "ue", "ur", "ut", "un", "um", "uz", "ux", "uc", "uv", "ub",
            "ip", "iq", "iw", "ie", "ir", "it", "in", "im", "iz", "ix", "ic", "iv", "ib",
            "op", "oq", "ow", "oe", "or", "ot", "on", "om", "oz", "ox", "oc", "ov", "ob",
            "hhp"
        ];
        let count = expected_hints.len();
        let mut hints = Hints::new(count, "hjklasdfgyuiopqwertnmzxcvb");
        let mut hint_texts = vec![];
        for _ in 0 .. count {
            hint_texts.push(hints.generate());
        }
        assert_eq!(expected_hints, hint_texts);
    }
}
