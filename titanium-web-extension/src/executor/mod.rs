/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
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

macro_rules! get_document {
    ($_self:ident) => {{
        let document = $_self.model.page.get_dom_document();
        if let Some(document) = document {
            document
        }
        else {
            return;
        }
    }};
}

mod scroll;

use std::collections::HashMap;

use glib::Cast;
use relm_state::{Relm, Update, UpdateNew};
use webkit2gtk_webextension::{
    DOMDOMSelectionExt,
    DOMDOMWindowExt,
    DOMDocumentExt,
    DOMElement,
    DOMElementExt,
    DOMHTMLElement,
    DOMHTMLElementExt,
    DOMHTMLInputElement,
    DOMHTMLInputElementExt,
    DOMHTMLSelectElement,
    DOMHTMLTextAreaElement,
    DOMNodeExt,
    WebPage,
    WebPageExt,
};

use titanium_common::{InnerMessage, PageId};
use titanium_common::Action::{
    self,
    FileInput,
    GoInInsertMode,
    NoAction,
};
use titanium_common::InnerMessage::*;

use dom::{
    NodeIter,
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
use self::Msg::*;

pub struct Executor {
    model: Model,
}

pub struct Model {
    activated_file_input: Option<DOMHTMLInputElement>,
    hint_keys: String,
    hint_map: HashMap<String, DOMElement>,
    last_hovered_element: Option<DOMElement>,
    page: WebPage,
    relm: Relm<Executor>,
    scroll_element: Option<DOMElement>,
}

#[derive(Msg)]
pub enum Msg {
    InitScrollElement,
    MessageRecv(InnerMessage),
    ServerSend(PageId, InnerMessage),
}

impl Update for Executor {
    type Model = Model;
    type ModelParam = WebPage;
    type Msg = Msg;

    fn model(relm: &Relm<Self>, page: WebPage) -> Model {
        Model {
            activated_file_input: None,
            hint_keys: String::new(),
            hint_map: HashMap::new(),
            last_hovered_element: None,
            page,
            relm: relm.clone(),
            scroll_element: None,
        }
    }

    fn update(&mut self, message: Msg) {
        match message {
            InitScrollElement => self.init_scroll_element(),
            MessageRecv(msg) =>
                match msg {
                    ActivateHint(follow_mode) => self.activate_hint(&follow_mode),
                    ActivateSelection() => self.activate_selection(),
                    EnterHintKey(key) => self.enter_hint_key(key),
                    FocusInput() => self.focus_input(),
                    GetCredentials() => self.send_credentials(),
                    GetScrollPercentage() => self.send_scroll_percentage(),
                    HideHints() => self.hide_hints(),
                    LoadUsernamePass(username, password) => self.load_username_pass(&username, &password),
                    ResetScrollElement() => self.model.scroll_element = None,
                    ScrollBottom() => self.scroll_bottom(),
                    ScrollBy(pixels) => self.scroll_by(pixels),
                    ScrollByX(pixels) => self.scroll_by_x(pixels),
                    ScrollTop() => self.scroll_top(),
                    SelectFile(file) => self.select_file(&file),
                    ShowHints(hint_chars) => self.show_hints(&hint_chars),
                    SubmitLoginForm() => self.submit_login_form(),
                    _ => warn!("Unexpected message received: {:?}", msg),
                },
            // To be listened by the user.
            ServerSend(_, _) => (),
        }
    }
}

impl UpdateNew for Executor {
    fn new(_relm: &Relm<Self>, model: Model) -> Self {
        Executor {
            model,
        }
    }
}

impl Executor {
    // Activate (click, focus, hover) the selected hint.
    fn activate_hint(&mut self, follow_mode: &str) {
        let follow =
            if follow_mode == "hover" {
                Executor::hover
            }
            else {
                Executor::click
            };

        let element = self.model.hint_map.get(&self.model.hint_keys)
            .and_then(|element| element.clone().downcast::<DOMHTMLElement>().ok());
        match element {
            Some(element) => {
                self.hide_hints();
                self.model.hint_map.clear();
                self.model.hint_keys.clear();
                let action = follow(self, element);
                self.send(ActivateAction(action));
            },
            None => self.send(ActivateAction(NoAction)),
        }
    }

    // Click on the link of the selected text.
    fn activate_selection(&self) {
        // TODO: switch to using some macros to simplify this code.
        let result = self.model.page.get_dom_document()
            .and_then(|document| document.get_default_view())
            .and_then(|window| window.get_selection())
            .and_then(|selection| selection.get_anchor_node())
            .and_then(|anchor_node| anchor_node.get_parent_element())
            .and_then(|parent| parent.downcast::<DOMHTMLElement>().ok());
        if let Some(parent) = result {
            parent.click();
        }
    }

    fn click(&mut self, element: DOMHTMLElement) -> Action {
        if let Ok(input_element) = element.clone().downcast::<DOMHTMLInputElement>() {
            let input_type = input_element.get_input_type().unwrap_or_default();
            match input_type.as_ref() {
                "button" | "checkbox" | "image" | "radio" | "reset" | "submit" => element.click(),
                // FIXME: file and color not opening.
                "color" => (),
                "file" => {
                    self.model.activated_file_input = Some(input_element);
                    return FileInput;
                },
                _ => {
                    element.focus();
                    return GoInInsertMode;
                },
            }
        }
        else if element.is::<DOMHTMLTextAreaElement>() {
            element.focus();
            return GoInInsertMode;
        }
        else if element.is::<DOMHTMLSelectElement>() {
            if element.get_attribute("multiple").is_some() {
                element.focus();
                return GoInInsertMode;
            }
            else {
                mouse_down(&element.upcast());
            }
        }
        else {
            element.click();
        }
        NoAction
    }

    // Handle the key press event for the hint mode.
    // This hides the hints that are not relevant anymore.
    fn enter_hint_key(&mut self, key: char) {
        self.model.hint_keys.push(key);
        let element = self.model.hint_map.get(&self.model.hint_keys)
            .and_then(|element| element.clone().downcast::<DOMHTMLElement>().ok());
        // If no element is found, hide the unrelevant hints.
        if element.is_some() {
            self.send(ClickHintElement());
        }
        else {
            let document = self.model.page.get_dom_document();
            if let Some(document) = document {
                let all_hidden = hide_unrelevant_hints(&document, &self.model.hint_keys);
                if all_hidden {
                    self.model.hint_keys.clear();
                    show_all_hints(&document);
                }
            }
        }
    }

    // Focus the first input element.
    fn focus_input(&mut self) {
        let document = self.model.page.get_dom_document();
        if let Some(document) = document {
            let tag_names = ["input", "textarea"];
            for tag_name in &tag_names {
                let iter = NodeIter::new(document.get_elements_by_tag_name(tag_name));
                for element in iter {
                    let tabindex = element.get_attribute("tabindex");
                    if !is_hidden(&document, &element) && is_enabled(&element) && is_text_input(&element)
                        && tabindex != Some("-1".to_string())
                    {
                        element.focus();
                        self.send(EnterInsertMode());
                        break;
                    }
                }
            }
        }
    }

    // Hide all the hints.
    fn hide_hints(&self) {
        let elements =
            self.model.page.get_dom_document()
            .and_then(|document| document.get_element_by_id(HINTS_ID))
            .and_then(|hints| get_body(&self.model.page).map(|body| (hints, body)));
        if let Some((hints, body)) = elements {
            check_err!(body.remove_child(&hints));
        }
    }

    fn hover(&mut self, element: DOMHTMLElement) -> Action {
        if let Some(ref element) = self.model.last_hovered_element {
            mouse_out(element);
        }
        self.model.last_hovered_element = Some(element.clone().upcast());
        mouse_over(&element.upcast());
        NoAction
    }

    // Load the username and the password in the login form.
    fn load_username_pass(&self, username: &str, password: &str) {
        let document = get_document!(self);
        load_username(&document, username);
        load_password(&document, password);
    }

    // Set the selected file on the input[type="file"].
    fn select_file(&mut self, file: &str) {
        if let Some(ref input_file) = self.model.activated_file_input.take() {
            // FIXME: this is not working.
            input_file.set_value(file);
        }
    }

    fn send(&self, message: InnerMessage) {
        self.model.relm.stream().emit(ServerSend(self.model.page.get_id(), message));
    }

    // Get the username and password from the login form.
    fn send_credentials(&mut self) {
        let mut username = String::new();
        let mut password = String::new();
        let credential =
            self.model.page.get_dom_document()
            .and_then(|document| get_credentials(&document));
        if let Some(credential) = credential {
            username = credential.username;
            password = credential.password;
        }
        // TODO: Send None instead of empty strings.
        self.send(Credentials(username, password));
    }

    // Get the page scroll percentage.
    fn send_scroll_percentage(&mut self) {
        let percentage = self.scroll_percentage();
        // TODO: only send the message if the percentage actually changed?
        // TODO: even better: add an even handler for scrolling and only send a message when an
        // actual scroll happened.
        self.send(ScrollPercentage(percentage));
    }

    // Show the hint of elements using the hint characters.
    // TODO: only send the hint characters once, not every time?
    fn show_hints(&mut self, hint_chars: &str) {
        self.model.hint_keys.clear();
        let body = wtry_opt_no_ret!(get_body(&self.model.page));
        let document = wtry_opt_no_ret!(self.model.page.get_dom_document());
        let (hints, hint_map) = wtry_opt_no_ret!(create_hints(&document, hint_chars));
        self.model.hint_map = hint_map;
        check_err!(body.append_child(&hints));
    }

    // Submit the login form.
    fn submit_login_form(&self) {
        let document = get_document!(self);
        submit_login_form(&document);
    }
}
