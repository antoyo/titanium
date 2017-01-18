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

//! Password management.

use std::collections::HashMap;

use glib::error;
use secret::{Collection, Item, PasswordError, Schema, Service};
use secret::SchemaAttributeType::{self, Boolean};

use app::AppResult;
use urls::base_url;

const KEYRING_NAME: &'static str = "Titanium Passwords";

/// A password manager is used to add, get and remove credentials.
pub struct PasswordManager {
    collection: Option<Collection>,
    schema: Schema,
}

impl PasswordManager {
    /// Create a new password manager.
    pub fn new() -> Self {
        let schema = Schema::new("com.titanium.Passwords", hash! {
            check => Boolean,
            url => SchemaAttributeType::String,
            username => SchemaAttributeType::String,
        });
        PasswordManager {
            collection: None,
            schema: schema,
        }
    }

    /// Add a credential.
    /// Returns true if the credential was added.
    pub fn add(&mut self, url: &str, username: &str, password: &str, check: bool) {
        if let Some(url) = base_url(url) {
            let check = false; // TODO
            let attributes =
                if check {
                    str_hash! {
                        check => check,
                        url => url,
                        username => username,
                    }
                }
                else {
                    str_hash! {
                        url => url,
                        username => username,
                    }
                };
            // TODO: handle errors.
            if let Some(ref collection) = self.collection {
                collection.item_create(&self.schema,
                    &format!("Password for {} on {}", username, url), password, &attributes, |_|
                {
                    // TODO: show an error if any.
                });
            }
        }
        else {
            warn!("Not adding the credentials for {}", url);
        }
    }

    fn assign_collection(&mut self, collection: Result<Collection, error::Error>) {
        // TODO: handle error.
        self.collection = Some(collection.unwrap());
    }

    fn create_keyring_if_needed(&mut self, result: Result<bool, error::Error>, service: Service) {
        if let Ok(true) = result {
            let mut exists = false;
            let collections = service.get_collections();
            for collection in collections {
                if collection.get_label() == Some(KEYRING_NAME.to_string()) {
                    exists = true;
                    self.collection = Some(collection);
                }
            }

            if !exists {
                // TODO: handle error.
                connect_static!(Collection, create[KEYRING_NAME](collection), self, assign_collection(collection));
            }
        }
    }

    /// Delete a password.
    /// Returns true if a credential was deleted.
    pub fn delete(&mut self, url: &str, username: &str) {
        if let Some(url) = base_url(url) {
            // TODO: handle error.
            self.get_one(str_hash! {
                url => url,
                username => username,
            }, |item| {
                item.delete(|_| {});
                // TODO: show an info.
                // TODO: show an error if any.
            });
        }
        else {
            warn!("Not deleting the password for {}", url);
        }
    }

    /// Search for items in the keyring, returning the first one.
    fn get_one<F: Fn(Item) + 'static>(&self, attributes: HashMap<String, String>, callback: F) {
        if let Some(ref collection) = self.collection {
            collection.search(&self.schema, &attributes, move |items| {
                if let Ok(mut items) = items {
                    if !items.is_empty() {
                        callback(items.remove(0));
                    }
                }
            });
        }
    }

    /// Get the usernames for a `url`.
    pub fn get_usernames<F: Fn(Vec<String>) + 'static>(&self, url: &str, callback: F) {
        if let Some(url) = base_url(url) {
            if let Some(ref collection) = self.collection {
                collection.search(&self.schema, &str_hash! {
                    url => url,
                }, move |items| {
                    // TODO: handle error.
                    if let Ok(items) = items {
                        let mut usernames = vec![];
                        for item in items {
                            let attributes = item.get_attributes();
                            if let Some(username) = attributes.get("username") {
                                usernames.push(username.clone());
                            }
                        }
                        callback(usernames);
                    }
                });
            }
        }
        else {
            warn!("Cannot get the usernames for {}", url);
        }
    }

    /// Get the password for a `url` and username.
    pub fn get_password<F: Fn(Result<String, PasswordError>) + 'static>(&self, url: &str, username: &str, callback: F) {
        if let Some(url) = base_url(url) {
            self.get_one(str_hash! {
                url => url,
                username => username,
            }, move |item| {
                if let Some(password) = item.get_secret().and_then(|secret| secret.get_text()) {
                    callback(Ok(password))
                }
            });
        }
        else {
            warn!("Cannot get the password for {}", url);
        }
    }

    pub fn init(&mut self, service: Service) {
        // TODO: only init when saving the first password.
        connect!(service.clone(), load_collections(loaded), self, create_keyring_if_needed(loaded, service));
    }
}
