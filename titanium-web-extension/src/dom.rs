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


use webkit2gtk_webextension::{DOMDocument, DOMDocumentExt, DOMDOMWindowExtManual, DOMElement, DOMElementExt, DOMHTMLElement, WebPage};

#[derive(Debug)]
pub struct Pos {
    pub x: i64,
    pub y: i64,
}

/// Get the body element of the web page.
pub fn get_body(page: &WebPage) -> Option<DOMHTMLElement> {
    page.get_dom_document().and_then(|document|
        document.get_body()
    )
}

/// Get the document element of the web page.
pub fn get_document(page: &WebPage) -> Option<DOMElement> {
    page.get_dom_document().and_then(|document|
        document.get_document_element()
    )
}

/// Get the position of an element relative to the screen.
pub fn get_offset(element: &DOMElement) -> Pos {
    let mut top = 0;
    let mut left = 0;
    let mut element = Some(element.clone());
    while let Some(el) = element {
        left += el.get_offset_left() as i64 - el.get_scroll_left();
        top += el.get_offset_top() as i64 - el.get_scroll_top();
        element = el.get_offset_parent();
    }
    Pos {
        x: left,
        y: top,
    }
}

/// Get the position of an element relative to the page root.
pub fn get_position(document: &DOMDocument, element: &DOMElement) -> Pos {
    let (left, top) =
        if let Some(body) = document.get_body() {
            let left = body.get_scroll_left();
            let top = body.get_scroll_top();
            (left, top)
        }
        else {
            (0, 0)
        };
    let mut pos = get_offset(element);
    pos.x += left;
    pos.y += top;
    pos
}

/// Hide an element.
pub fn hide(element: &DOMElement) {
    if let Some(style) = element.get_style() {
        style.set_property("display", "none", "").ok();
    }
}

/// Check if an element is visible.
pub fn is_visible(document: &DOMDocument, element: &DOMElement) -> bool {
    if let Some(window) = document.get_default_view() {
        if let Some(document_element) = document.get_document_element() {
            let height = document_element.get_client_height() as i64;
            let width = document_element.get_client_width() as i64;
            let pos = get_offset(element);
            if pos.x < 0 || pos.x > width || pos.y < 0 || pos.y > height {
                return false;
            }
            let mut element = Some(element.clone());
            while let Some(el) = element {
                if el.get_tag_name() == Some("BODY".to_string()) {
                    return true;
                }
                if let Some(style) = window.get_computed_style(&el, None) {
                    if style.get_property_value("display") == Some("none".to_string()) ||
                        style.get_property_value("visibility") == Some("hidden".to_string()) ||
                        style.get_property_value("opacity") == Some("0".to_string())
                    {
                        return false;
                    }
                }
                element = el.get_offset_parent();
            }
        }
    }
    false
}
