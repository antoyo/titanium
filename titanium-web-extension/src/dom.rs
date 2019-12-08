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

use glib::{Cast, ObjectExt};
use regex::Regex;

use webkit2gtk_webextension::{
    DOMClientRectExt,
    DOMClientRectListExt,
    DOMCSSStyleDeclarationExt,
    DOMDocument,
    DOMDocumentExt,
    DOMDOMWindowExt,
    DOMElement,
    DOMElementExt,
    DOMEventExt,
    DOMEventTarget,
    DOMEventTargetExt,
    DOMHTMLAnchorElement,
    DOMHTMLAnchorElementExt,
    DOMHTMLButtonElement,
    DOMHTMLButtonElementExt,
    DOMHTMLCollection,
    DOMHTMLCollectionExt,
    DOMHTMLElement,
    DOMHTMLFieldSetElement,
    DOMHTMLFieldSetElementExtManual,
    DOMHTMLFrameElement,
    DOMHTMLFrameElementExt,
    DOMHTMLIFrameElement,
    DOMHTMLIFrameElementExt,
    DOMHTMLInputElement,
    DOMHTMLInputElementExt,
    DOMHTMLSelectElement,
    DOMHTMLSelectElementExt,
    DOMHTMLTextAreaElement,
    DOMHTMLTextAreaElementExt,
    DOMMouseEvent,
    DOMMouseEventExt,
    DOMNode,
    DOMNodeExt,
    DOMNodeList,
    DOMNodeListExt,
    WebPage,
    WebPageExt,
};

macro_rules! return_if_disabled {
    ($ty:ty, $element:expr) => {
        if $element.is::<$ty>() {
            if let Ok(element) = $element.clone().downcast::<$ty>() {
                if element.get_disabled() {
                    return false;
                }
            }
        }
    };
}

macro_rules! iter {
    ($name:ident, $list:ident) => {
        /// A `DOMElement` iterator for a node list.
        pub struct $name {
            index: u64,
            node_list: Option<$list>,
        }

        impl $name {
            /// Create a new dom element iterator.
            pub fn new(node_list: Option<$list>) -> Self {
                $name {
                    index: 0,
                    node_list,
                }
            }

            pub fn len(&self) -> usize {
                self.node_list.as_ref().map(|list| list.get_length() as usize)
                    .unwrap_or(0)
            }
        }

        impl Iterator for $name {
            type Item = DOMElement;

            fn next(&mut self) -> Option<Self::Item> {
                match self.node_list {
                    Some(ref list) => {
                        if self.index < list.get_length() {
                            let element = list.item(self.index);
                            self.index += 1;
                            element.and_then(|element| element.downcast::<DOMElement>().ok())
                        }
                        else {
                            None
                        }
                    },
                    None => None,
                }
            }
        }
    };
}

iter!(NodeIter, DOMNodeList);
iter!(ElementIter, DOMHTMLCollection);

#[derive(Debug)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

/// Trigger a click event on the element.
pub fn click(element: &DOMElement, ctrl_key: bool) {
    mouse_event("click", element, ctrl_key);
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

pub fn get_elements_by_tag_name_in_all_frames(document: &DOMDocument, tag_name: &str) -> Vec<(DOMDocument, DOMElement)> {
    let mut elements = vec![];
    let iter = NodeIter::new(document.get_elements_by_tag_name(tag_name));
    for element in iter {
        elements.push((document.clone(), element));
    }

    let iter = NodeIter::new(document.get_elements_by_tag_name("frame"));
    for frame in iter {
        if let Ok(frame) = frame.downcast::<DOMHTMLFrameElement>() {
            if let Some(document) = frame.get_content_document() {
                let iter = NodeIter::new(document.get_elements_by_tag_name(tag_name));
                for element in iter {
                    elements.push((document.clone(), element));
                }
            }
        }
    }

    let iter = NodeIter::new(document.get_elements_by_tag_name("iframe"));
    for frame in iter {
        if let Ok(frame) = frame.downcast::<DOMHTMLIFrameElement>() {
            if let Some(document) = frame.get_content_document() {
                let iter = NodeIter::new(document.get_elements_by_tag_name(tag_name));
                for element in iter {
                    elements.push((document.clone(), element));
                }
            }
        }
    }

    elements
}

/// Get the body if it exists or the html element.
pub fn get_hints_container(page: &WebPage) -> Option<DOMNode> {
    let document = page.get_dom_document()?;
    if let Some(elements) = document.get_elements_by_tag_name("body") {
        if let Some(body) = elements.item(0) {
            return Some(body);
        }
    }
    let document = get_document(page)?;
    Some(document.upcast::<DOMNode>())
}

/// Get the href attribute of an anchor element.
pub fn get_href(element: &DOMHTMLElement) -> Option<String> {
    if let Ok(input_element) = element.clone().downcast::<DOMHTMLAnchorElement>() {
        input_element.get_href().map(Into::into)
    }
    else {
        None
    }
}

fn get_frame_offsets(element: &DOMElement) -> Option<Pos> {
    let element = element.clone().upcast::<DOMNode>();
    let document = element.get_owner_document()?;
    let mut window = document.get_default_view();
    let mut previous_window = None;
    let mut x = 0.0;
    let mut y = 0.0;
    while window != previous_window {
        if let Some(frame) = window.as_ref().and_then(|window| window.get_frame_element()) {
            let rects = frame.get_client_rects()?;
            let rect = rects.item(0)?;
            x += rect.get_left();
            y += rect.get_top();
        }
        previous_window = window.clone();
        let new_window =
            if let Some(ref win) = window {
                win.get_parent()
            }
            else {
                break;
            };
        window = new_window;
    }
    Some(Pos {
        x,
        y,
    })
}

/// Get the position of an element relative to the page root.
pub fn get_position(element: &DOMElement) -> Option<Pos> {
    let rects = element.get_client_rects()?;
    let rect = rects.item(0)?;
    let frame_offsets = get_frame_offsets(element).unwrap_or(Pos { x: 0.0, y: 0.0 });

    let document = element.get_owner_document()?;
    let window = document.get_default_view()?;
    let scroll_x = window.get_scroll_x();
    let scroll_y = window.get_scroll_y();
    Some(Pos {
        x: rect.get_left() + frame_offsets.x + scroll_x as f32,
        y: rect.get_top() + frame_offsets.y + scroll_y as f32,
    })
}

/// Hide an element.
pub fn hide(element: &DOMElement) {
    let style = wtry_opt_no_ret!(element.get_style());
    wtry!(DOMCSSStyleDeclarationExt::set_property(&style, "display", "none", ""));
}

/// Check if an input element is enabled.
/// Other element types return true.
pub fn is_enabled(element: &DOMElement) -> bool {
    let is_form_element =
        element.is::<DOMHTMLButtonElement>() ||
        element.is::<DOMHTMLInputElement>() ||
        element.is::<DOMHTMLSelectElement>() ||
        element.is::<DOMHTMLTextAreaElement>();
    if is_form_element {
        let mut element = Some(element.clone());
        while let Some(el) = element {
            if el.get_tag_name().map(Into::into) == Some("BODY".to_string()) {
                break;
            }
            return_if_disabled!(DOMHTMLButtonElement, el);
            return_if_disabled!(DOMHTMLInputElement, el);
            return_if_disabled!(DOMHTMLSelectElement, el);
            return_if_disabled!(DOMHTMLTextAreaElement, el);
            return_if_disabled!(DOMHTMLFieldSetElement, el);
            element = el.get_parent_element();
        }
    }
    true
}

/// Check if an element is hidden.
/// This is not exactly the opposite as `is_visible` since `is_hidden` returns false for elements that
/// are visible, but outside the viewport.
pub fn is_hidden(document: &DOMDocument, element: &DOMElement) -> bool {
    let window = unwrap_opt_or_ret!(document.get_default_view(), true);
    let mut element = Some(element.clone());
    while let Some(el) = element {
        if el.get_tag_name().map(Into::into) == Some("BODY".to_string()) {
            return false;
        }
        let style = unwrap_opt_or_ret!(window.get_computed_style(&el, None), true);
        if style.get_property_value("display").map(Into::into) == Some("none".to_string()) ||
            style.get_property_value("visibility").map(Into::into) == Some("hidden".to_string()) ||
            style.get_property_value("opacity").map(Into::into) == Some("0".to_string())
        {
            return true;
        }
        element = el.get_parent_element();
    }
    true
}

/// Check if an element is a text input element (including all its variant like number, tel,
/// search, …).
pub fn is_text_input(element: &DOMElement) -> bool {
    let input_type = element.clone().downcast::<DOMHTMLInputElement>().ok()
        .and_then(|input_element| input_element.get_input_type())
        .map(Into::into)
        .unwrap_or_else(|| "text".to_string());
    match input_type.as_ref() {
        "button" | "checkbox" | "color" | "file" | "hidden" | "image" | "radio" | "reset" | "submit" => false,
        _ => true,
    }
}

/// Check if an element is visible and in the viewport.
pub fn is_visible(document: &DOMDocument, element: &DOMElement) -> bool {
    let window = unwrap_opt_or_ret!(document.get_default_view(), false);
    let rect = unwrap_opt_or_ret!(element.get_bounding_client_rect(), false);
    let x1 = rect.get_left();
    let x2 = rect.get_right();
    let y1 = rect.get_top();
    let y2 = rect.get_bottom();

    let height = window.get_inner_height() as f32;
    let width = window.get_inner_width() as f32;
    (x1 >= 0.0 || x2 >= 0.0) && x1 < width && (y1 >= 0.0 || y2 >= 0.0) && y1 < height
}

/// Trigger a mouse down event on the element.
pub fn mouse_down(element: &DOMElement) {
    mouse_event("mousedown", element, false);
}

/* TODO: delete.
/// Trigger a mouse enter event on the element.
pub fn mouse_enter(element: &DOMElement) {
    mouse_event("mouseenter", element);
}*/

/// Trigger a change event on the element.
pub fn change_event(element: &DOMElement) {
    let event = wtry_opt_no_ret!(element.get_owner_document()
        .and_then(|document| document.create_event("HTMLEvents").ok()));
    event.init_event("change", false, true);
    let element: DOMEventTarget = element.clone().upcast();
    wtry!(element.dispatch_event(&event));
}

/// Trigger a mouse event on the element.
pub fn mouse_event(event_name: &str, element: &DOMElement, ctrl_key: bool) {
    let event = wtry_opt_no_ret!(element.get_owner_document()
        .and_then(|document| document.create_event("MouseEvents").ok()));
    let window = wtry_opt_no_ret!(element.get_owner_document()
        .and_then(|document| document.get_default_view()));
    let event = wtry_no_show!(event.downcast::<DOMMouseEvent>());
    // TODO: use the previously hovered element for the last parameter.
    event.init_mouse_event(event_name, true, true, &window, 0, 0, 0, 0, 0, ctrl_key, false, false, false, 0, element);
    let element: DOMEventTarget = element.clone().upcast();
    wtry!(element.dispatch_event(&event));
}

/// Trigger a mouse out event on the element.
pub fn mouse_out(element: &DOMElement) {
    mouse_event("mouseout", element, false);
}

/// Trigger a mouse over event on the element.
pub fn mouse_over(element: &DOMElement) {
    mouse_event("mouseover", element, false);
}

/// Show an element.
pub fn show(element: &DOMElement) {
    let style = wtry_opt_no_ret!(element.get_style());
    wtry!(style.remove_property("display"));
}

/// Lookup dom elements by tag and regex
pub fn match_pattern(document: &DOMDocument, selector: &str, regex: Regex) -> Option<DOMElement> {
    let iter = NodeIter::new(document.get_elements_by_tag_name(selector));

    for element in iter {
        if let Some(text) = element.get_inner_html() {
            if regex.is_match(&text) {
                return Some(element);
            }
        }
    }

    None
}
