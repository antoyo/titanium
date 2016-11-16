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

use glib::Cast;
use webkit2gtk_webextension::{
    DOMDocument,
    DOMDocumentExt,
    DOMElementExt,
    DOMHTMLFormElement,
    DOMHTMLInputElement,
    DOMNodeExt,
};

use dom::{ElementIter, is_hidden};

pub struct Credential {
    pub check: bool,
    pub password: String,
    pub username: String,
}

/// Find a login form.
/// If a visible form exists, prefer it.
fn find_login_form(document: &DOMDocument) -> Option<DOMHTMLFormElement> {
    let elements = ElementIter::new(document.query_selector_all("input[type='password']").ok());
    for input_element in elements {
        if !is_hidden(document, &input_element) {
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
    let login_form = get_login_form(document);
    if let Some(login_form) = login_form {
        let username_element = login_form.query_selector("input[type='text']").ok()
            .and_then(|element| element.downcast::<DOMHTMLInputElement>().ok());
        if let Some(element) = username_element {
            username = element.get_value().unwrap_or_default();
        }
        let password_element = login_form.query_selector("input[type='password']").ok()
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
                if let Ok(form) = form.downcast() {
                    return Some(form);
                }
            }
            None
        })
}

/// Load the password in the login form.
pub fn load_password(document: &DOMDocument, password: &str) {
    let password_input =
        find_login_form(document)
            .and_then(|login_form| login_form.query_selector("input[type='password']").ok())
            .and_then(|element| element.downcast::<DOMHTMLInputElement>().ok());
    if let Some(password_input) = password_input {
        password_input.set_value(password);
    }
}

/// Load the username in the login form.
pub fn load_username(document: &DOMDocument, username: &str) {
    let username_input =
        find_login_form(document)
            .and_then(|login_form| login_form.query_selector("input[type='text']").ok())
            .and_then(|element| element.downcast::<DOMHTMLInputElement>().ok());
    if let Some(username_input) = username_input {
        username_input.set_value(username);
    }
}

/// Submit the login form.
pub fn submit_login_form(document: &DOMDocument) {
    let login_form = find_login_form(document);
    if let Some(login_form) = login_form {
        login_form.submit();
    }
}
