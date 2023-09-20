/*
 * Copyright (c) 2017-2018 Boucher, Antoni <bouanto@zoho.com>
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
        let document = $_self.model.page.dom_document();
        if let Some(document) = document {
            document
        }
        else {
            return;
        }
    }};
}

mod marks;
mod scroll;

use std::collections::HashMap;
use std::f32;
use std::sync::Mutex;

use gio::Cancellable;
use glib::{Cast, Closure, ObjectExt, ToVariant};
use regex::Regex;
use relm::{Relm, Update, UpdateNew};
use webkit2gtk_webextension::{
    traits::{
        DOMDocumentExt,
        DOMDOMSelectionExt,
        DOMDOMWindowExt,
        DOMElementExt,
        DOMEventTargetExt,
        DOMHTMLElementExt,
        DOMHTMLInputElementExt,
        DOMNodeExt,
        WebPageExt,
    },
    DOMElement,
    DOMHTMLElement,
    DOMHTMLInputElement,
    DOMHTMLSelectElement,
    DOMHTMLTextAreaElement,
    UserMessage,
    WebPage,
};

use titanium_common::{FollowMode, InnerMessage, protocol::encode};
use titanium_common::Action::{
    self,
    CopyLink,
    DownloadLink,
    FileInput,
    GoInInsertMode,
    NoAction,
};
use titanium_common::InnerMessage::*;

use dom::{
    get_body,
    get_elements_by_tag_name_in_all_frames,
    get_hints_container,
    get_href,
    get_position,
    is_enabled,
    is_hidden,
    is_text_input,
    mouse_down,
    click,
    mouse_out,
    mouse_over,
    match_pattern,
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
    marks: HashMap<u8, u32>, // Byte to percent.
    page: WebPage,
    relm: Relm<Executor>,
    scroll_element: Option<DOMElement>,
}

#[derive(Msg)]
pub enum Msg {
    DocumentLoaded,
    MessageRecv(InnerMessage),
    Scroll,
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
            marks: HashMap::new(),
            page,
            relm: relm.clone(),
            scroll_element: None,
        }
    }

    fn update(&mut self, message: Msg) {
        match message {
            DocumentLoaded => {
                self.init_scroll_element();
                self.send_scroll_percentage();

                let stream = self.model.relm.stream().clone();
                let stream = Mutex::new(::send_cell::SendCell::new(stream));
                let handler = Closure::new(move |_| {
                    let stream = stream.lock().unwrap();
                    stream.get().emit(Scroll);
                    None
                });

                if self.model.scroll_element == get_body(&self.model.page).map(|el| el.upcast()) {
                    let document = wtry_opt_no_ret!(self.model.page.dom_document());
                    document.add_event_listener_with_closure("scroll", &handler, false);
                }
                else {
                    let element = self.model.scroll_element.as_ref().unwrap();
                    element.add_event_listener_with_closure("scroll", &handler, false);
                }
            },
            MessageRecv(msg) =>
                match msg {
                    ActivateHint(follow_mode, ctrl_key) => self.activate_hint(follow_mode, ctrl_key),
                    ActivateSelection() => self.activate_selection(),
                    ClickNextPage() => self.click_next_page(),
                    ClickPrevPage() => self.click_prev_page(),
                    EnterHintKey(key) => self.enter_hint_key(key),
                    FocusInput() => self.focus_input(),
                    GetCredentials() => self.send_credentials(),
                    GoToMark(mark) => self.go_to_mark(mark),
                    HideHints() => self.hide_hints(),
                    InsertText(text) => self.insert_text(&text),
                    LoadUsernamePass(username, password) => self.load_username_pass(&username, &password),
                    Mark(char) => self.add_mark(char),
                    ResetMarks() => self.reset_marks(),
                    ResetScrollElement() => self.reset_scroll_element(),
                    ScrollBy(pixels) => self.scroll_by(pixels),
                    ScrollByX(pixels) => self.scroll_by_x(pixels),
                    ScrollTop() => self.scroll_top(),
                    ScrollToPercent(percent) => self.scroll_to_percent(percent),
                    SelectFile(file) => self.select_file(&file),
                    ShowHints(hint_chars) => self.show_hints(&hint_chars),
                    SubmitLoginForm() => self.submit_login_form(),
                    _ => warn!("Unexpected message received: {:?}", msg),
                },
            Scroll => self.send_scroll_percentage(),
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
    fn activate_hint(&mut self, follow_mode: FollowMode, ctrl_key: bool) {
        let element = self.model.hint_map.get(&self.model.hint_keys)
            .and_then(|element| element.clone().downcast::<DOMHTMLElement>().ok());
        match element {
            Some(element) => {
                self.hide_hints();
                self.model.hint_map.clear();
                self.model.hint_keys.clear();
                let action =
                    match follow_mode {
                        FollowMode::Click => self.click(element, ctrl_key),
                        FollowMode::CopyLink => self.copy_link(element),
                        FollowMode::Download => self.download_link(element),
                        FollowMode::Hover => self.hover(element),
                    };
                self.send(ActivateAction(action));
            },
            None => self.send(ActivateAction(NoAction)),
        }
    }

    // Click on the link of the selected text.
    fn activate_selection(&self) {
        // TODO: switch to using some macros to simplify this code.
        let result = self.model.page.dom_document()
            .and_then(|document| document.default_view())
            .and_then(|window| window.selection())
            .and_then(|selection| selection.anchor_node())
            .and_then(|anchor_node| anchor_node.parent_element())
            .and_then(|parent| parent.downcast::<DOMHTMLElement>().ok());
        if let Some(parent) = result {
            parent.click();
        }
    }

    fn click(&mut self, element: DOMHTMLElement, ctrl_key: bool) -> Action {
        if let Ok(input_element) = element.clone().downcast::<DOMHTMLInputElement>() {
            let input_type = input_element.input_type().map(|string| string.to_string()).unwrap_or_default();
            match input_type.as_ref() {
                "button" | "checkbox" | "image" | "radio" | "reset" | "submit" => {
                    click(&element.upcast(), ctrl_key);
                    NoAction
                },
                // FIXME: file and color not opening.
                "color" => NoAction,
                "file" => {
                    self.model.activated_file_input = Some(input_element);
                    FileInput
                },
                _ => {
                    element.focus();
                    GoInInsertMode
                },
            }
        }
        else if element.is::<DOMHTMLTextAreaElement>() {
            element.focus();
            GoInInsertMode
        }
        else if element.is::<DOMHTMLSelectElement>() {
            if element.attribute("multiple").is_some() {
                element.focus();
                GoInInsertMode
            }
            else {
                mouse_down(&element.upcast());
                NoAction
            }
        }
        else {
            click(&element.upcast(), ctrl_key);
            NoAction
        }
    }

    fn copy_link(&self, element: DOMHTMLElement) -> Action {
        let href = unwrap_opt_or_ret!(get_href(&element), NoAction);
        CopyLink(href)
    }

    fn click_next_page(&mut self) {
        let regex = Regex::new(r"(?i:next|forward|older|more|›|»)|(?:<.+>)>(?:<.+>)").unwrap();

        let document = get_document!(self);
        if let Some(link) = match_pattern(&document, "a", regex) {
            let element = wtry_no_show!(link.clone().downcast::<DOMHTMLElement>());
            element.click();
        }
        else {
            // TODO: Check if url (not text) is *very* similar to our current one
            // example.com/page/4 => example.com/page/5
            warn!("No next link found");
        }
    }

    fn click_prev_page(&mut self) {
        let regex = Regex::new(r"(?i:prev(ious)|back|newer|less|«|‹)|(?:<.+>)<(?:<.+>)").unwrap();

        let document = get_document!(self);
        if let Some(link) = match_pattern(&document, "a", regex) {
            let element = wtry_no_show!(link.clone().downcast::<DOMHTMLElement>());
            element.click();
        }
        else {
            // TODO: See above
            warn!("No previous link found");
        }
    }

    fn download_link(&self, element: DOMHTMLElement) -> Action {
        let href = unwrap_opt_or_ret!(get_href(&element), NoAction);
        DownloadLink(href)
    }

    // Handle the key press event for the hint mode.
    // This hides the hints that are not relevant anymore.
    fn enter_hint_key(&mut self, key: char) {
        self.model.hint_keys.push(key);
        let element = self.model.hint_map.get(&self.model.hint_keys)
            .and_then(|element| element.clone().downcast::<DOMHTMLElement>().ok());
        // If no element is found, hide the unrelevant hints.
        match element {
            Some(element) => {
                // TODO: perhaps it'd involve less message if we remove the ActivateHint message.
                self.send(ClickHintElement(get_href(&element)));
            },
            _ => {
                let document = self.model.page.dom_document();
                if let Some(document) = document {
                    let all_hidden = hide_unrelevant_hints(&document, &self.model.hint_keys);
                    if all_hidden {
                        self.model.hint_keys.clear();
                        show_all_hints(&document);
                    }
                }
            }
        }
    }

    // Focus the first input element.
    fn focus_input(&mut self) {
        let document = self.model.page.dom_document();
        if let Some(document) = document {
            let tag_names = ["input", "textarea"];
            let mut element_to_focus = None;
            let mut element_y_pos = f32::INFINITY;
            for tag_name in &tag_names {
                let iter = get_elements_by_tag_name_in_all_frames(&document, tag_name);
                for (document, element) in iter {
                    let tabindex = element.attribute("tabindex").map(Into::into);
                    if !is_hidden(&document, &element) && is_enabled(&element) && is_text_input(&element)
                        && tabindex != Some("-1".to_string())
                    {
                        if let Some(pos) = get_position(&element) {
                            // TODO: If y is equal, compare x?
                            if pos.y < element_y_pos {
                                element_y_pos = pos.y;
                                element_to_focus = Some(element);
                            }
                        }
                    }
                }
            }

            if let Some(element) = element_to_focus {
                element.focus();
                element.scroll_into_view_if_needed(false);
                self.send(EnterInsertMode());
            }
        }
    }

    // Hide all the hints.
    fn hide_hints(&self) {
        let elements =
            self.model.page.dom_document()
            .and_then(|document| document.element_by_id(HINTS_ID))
            .and_then(|hints| get_hints_container(&self.model.page).map(|container| (hints, container)));
        if let Some((hints, container)) = elements {
            check_err!(container.remove_child(&hints));
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

    fn insert_text(&self, text: &str) {
        let document = get_document!(self);
        let active_element = wtry_opt_no_ret!(document.active_element());
        let element = wtry_no_show!(active_element.downcast::<DOMHTMLInputElement>());
        element.set_value(text);
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
        let bytes =
            match encode(message) {
                Ok(message) => message,
                Err(error) => {
                    error!("{}", error);
                    return;
                },
            };
        let message = UserMessage::new("", Some(&bytes.to_variant()));
        self.model.page.send_message_to_view(&message, None::<&Cancellable>, |_| {});
    }

    // Get the username and password from the login form.
    fn send_credentials(&mut self) {
        let mut username = String::new();
        let mut password = String::new();
        let credential =
            self.model.page.dom_document()
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
        self.send(ScrollPercentage(percentage));
    }

    // Show the hint of elements using the hint characters.
    // TODO: only send the hint characters once, not every time?
    fn show_hints(&mut self, hint_chars: &str) {
        self.model.hint_keys.clear();
        let container = wtry_opt_no_ret!(get_hints_container(&self.model.page));
        let document = wtry_opt_no_ret!(self.model.page.dom_document());
        let (hints, hint_map) = wtry_opt_no_ret!(create_hints(&document, hint_chars));
        self.model.hint_map = hint_map;
        check_err!(container.append_child(&hints));
    }

    // Submit the login form.
    fn submit_login_form(&self) {
        let document = get_document!(self);
        submit_login_form(&document);
    }
}
