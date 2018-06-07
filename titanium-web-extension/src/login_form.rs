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

use glib::Cast;
use webkit2gtk_webextension::{
    DOMDocument,
    DOMDocumentExt,
    DOMElementExt,
    DOMHTMLFormElement,
    DOMHTMLFormElementExt,
    DOMHTMLInputElement,
    DOMHTMLInputElementExt,
    DOMNodeExt,
};

use dom::{
    NodeIter,
    change_event,
    is_hidden,
};
use option_util::OptionExt;

pub struct Credential {
    pub check: bool,
    pub password: String,
    pub username: String,
}

/// Find a login form.
/// If a visible form exists, prefer it.
fn find_login_form(document: &DOMDocument) -> Option<DOMHTMLFormElement> {
    let elements = NodeIter::new(document.query_selector_all("input[type='password']").ok());
    let elements_count = elements.len();
    for input_element in elements {
        // TODO: check that all elements are hidden instead of checking that there is only one
        // element.
        if !is_hidden(document, &input_element) || elements_count == 1 {
            let mut form_element = None;
            let mut element = Some(input_element);
            while let Some(el) = element {
                if el.get_tag_name().unwrap_or_default().to_lowercase() == "form" {
                    form_element = Some(el);
                    break;
                }
                element = el.get_parent_element();
            }
            if let Some(form) = form_element {
                if let Ok(form) = form.downcast() {
                    return Some(form);
                }
            }
        }
        // TODO: also look for invisible login form.
    }
    None
}

/// Find the credentials from the login form of the active element in the document.
pub fn get_credentials(document: &DOMDocument) -> Option<Credential> {
    let mut password = String::new();
    let mut username = String::new();
    if let Some(login_form) = get_login_form(document) {
        let username_element = login_form.query_selector("input[type='text']").flatten()
            .and_then(|element| element.downcast::<DOMHTMLInputElement>().ok());
        if let Some(element) = username_element {
            username = element.get_value().unwrap_or_default();
        }
        let password_element = login_form.query_selector("input[type='password']").flatten()
            .and_then(|element| element.downcast::<DOMHTMLInputElement>().ok());
        if let Some(element) = password_element {
            password = element.get_value().unwrap_or_default();
        }
    }
    if username.is_empty() || password.is_empty() {
        None
    }
    else {
        Some(Credential {
            check: false,
            password: password,
            username: username,
        })
    }
}

/// Get the login form.
fn get_login_form(document: &DOMDocument) -> Option<DOMHTMLFormElement> {
    document.get_active_element()
        .and_then(|active_element| {
            let mut form_element = None;
            let mut element = Some(active_element);
            while let Some(el) = element {
                if el.get_tag_name().unwrap_or_default().to_lowercase() == "form" {
                    form_element = Some(el);
                    break;
                }
                element = el.get_parent_element();
            }
            if let Some(form) = form_element {
                return form.downcast().ok();
            }
            None
        })
}

/// Load the password in the login form.
pub fn load_password(document: &DOMDocument, password: &str) {
    let password_inputs =
        find_login_form(document)
            .and_then(|login_form| login_form.query_selector_all("input[type='password']").ok());
    let inputs = NodeIter::new(password_inputs);
    for input in inputs {
        if !is_hidden(document, &input) {
            let password_input = input.clone().downcast::<DOMHTMLInputElement>();
            if let Ok(password_input) = password_input {
                password_input.set_value(password);
                change_event(&input);
                break;
            }
        }
    }
}

/// Load the username in the login form.
pub fn load_username(document: &DOMDocument, username: &str) {
    let username_inputs =
        find_login_form(document)
            .and_then(|login_form|
                // FIXME: check for more types than just text and email.
                login_form.query_selector_all("input[type='text'], input[type='email']").ok()
            );
    let inputs = NodeIter::new(username_inputs);
    for input in inputs {
        if !is_hidden(document, &input) {
            let username_input = input.clone().downcast::<DOMHTMLInputElement>();
            if let Ok(username_input) = username_input {
                username_input.set_value(username);
                change_event(&input);
                break;
            }
        }
    }
}

/// Submit the login form.
pub fn submit_login_form(document: &DOMDocument) {
    let login_form = find_login_form(document);
    if let Some(login_form) = login_form {
        login_form.submit();
    }
}
