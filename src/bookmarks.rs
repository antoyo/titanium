/*
 * Copyright (c) 2016-2017 Boucher, Antoni <bouanto@zoho.com>
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

//! Bookmark management.

use std::cell::RefCell;
use std::collections::HashSet;
use std::path::PathBuf;
use std::result;

use rusqlite::Connection;
use rusqlite::types::ToSql;

use errors::{Error, Result};

thread_local! {
    static CONNECTION: RefCell<Option<Connection>> = RefCell::new(None);
}

/// A bookmark has a title and a URL and optionally some tags.
#[derive(Debug)]
pub struct Bookmark {
    pub tags: String,
    pub title: String,
    pub url: String,
}

impl Bookmark {
    /// Create a new bookmark.
    pub fn new(url: String, title: String, tags: String) -> Self {
        Bookmark {
            tags: tags,
            title: title,
            url: url,
        }
    }
}

/// A bookmark manager is use to add, search and remove bookmarks.
pub struct BookmarkManager {
}

impl BookmarkManager {
    /// Create a new bookmark manager.
    pub fn new() -> Self {
        BookmarkManager {
        }
    }

    /// Add a bookmark.
    /// Returns true if the bookmark was added.
    pub fn add(&self, url: String, title: Option<String>) -> Result<bool> {
        CONNECTION.with(|connection| {
            if let Some(ref connection) = *connection.borrow() {
                if let Ok(inserted_count) = connection.execute("
                    INSERT INTO bookmarks (title, url)
                    VALUES ($1, $2)
                    ", &[&title.unwrap_or_default(), &url])
                {
                    return Ok(inserted_count > 0);
                }
            }
            Ok(false)
        })
    }

    /// Connect to the database if it is not already connected.
    pub fn connect(&self, filename: PathBuf) -> Result<()> {
        CONNECTION.with(|connection| {
            let mut connection = connection.borrow_mut();
            if connection.is_none() {
                let db = Connection::open(filename)?;
                // Activate foreign key contraints in SQLite.
                db.execute("PRAGMA foreign_keys = ON", &[])?;
                *connection = Some(db);
            }
            Ok(())
        })
    }

    /// Create the SQL tables for the bookmarks.
    pub fn create_tables(&self) -> Result<()> {
        CONNECTION.with(|connection| {
            if let Some(ref connection) = *connection.borrow() {
                connection.execute("
                CREATE TABLE IF NOT EXISTS bookmarks
                ( id INTEGER PRIMARY KEY
                , title TEXT NOT NULL
                , url TEXT NOT NULL UNIQUE
                , visit_count INTEGER NOT NULL DEFAULT 0
                )", &[])?;

                connection.execute("
                CREATE TABLE IF NOT EXISTS tags
                ( id INTEGER PRIMARY KEY
                , name TEXT NOT NULL UNIQUE
                )", &[])?;

                connection.execute("
                CREATE TABLE IF NOT EXISTS bookmarks_tags
                ( bookmark_id INTEGER NOT NULL
                , tag_id INTEGER NOT NULL
                , PRIMARY KEY (bookmark_id, tag_id)
                , FOREIGN KEY(bookmark_id) REFERENCES bookmarks(id) ON DELETE CASCADE
                , FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
                )", &[])?;
            }
            Ok(())
        })
    }

    /// Delete a bookmark.
    /// Returns true if a bookmark was deleted.
    pub fn delete(&self, url: &str) -> Result<bool> {
        CONNECTION.with(|connection| {
            if let Some(ref connection) = *connection.borrow() {
                if let Ok(deleted_count) = connection.execute("
                    DELETE FROM bookmarks
                    WHERE url = $1
                    ", &[&url.to_string()])
                {
                    return Ok(deleted_count > 0);
                }
            }
            Ok(false)
        })
    }

    /// Delete the tags that are in `original_tags` but not in `tags`.
    fn delete_tags(&self, connection: &Connection, bookmark_id: i32, original_tags: &[String], tags: &[String])
        -> Result<()>
    {
        let original_tags: HashSet<_> = original_tags.iter().collect();
        let tags: HashSet<_> = tags.iter().collect();
        let tags_to_delete = &original_tags - &tags;
        for tag in tags_to_delete {
            let tag_id = self.get_tag_id(connection, tag)?;
            connection.execute("
                DELETE FROM bookmarks_tags
                WHERE bookmark_id = $1 AND tag_id = $2
            ", &[&bookmark_id, &tag_id])?;
        }
        Ok(())
    }

    /// Check if a bookmark exists.
    pub fn exists(&self, url: &str) -> bool {
        self.get_id(url).is_some()
    }

    /// Get the id of a bookmark.
    pub fn get_id(&self, url: &str) -> Option<i32> {
        CONNECTION.with(|connection| {
            if let Some(ref connection) = *connection.borrow() {
                if let Ok(mut statement) = connection.prepare("
                        SELECT id
                        FROM bookmarks
                        WHERE url = $1
                    ")
                {
                    if let Ok(mut rows) = statement.query(&[&url.to_string()])
                    {
                        return rows.next().and_then(|row| row.ok().map(|row| row.get(0)));
                    }
                }
            }
            None
        })
    }

    /// Get the tag ID of a bookmark.
    pub fn get_tag_id(&self, connection: &Connection, tag: &str) -> Result<i32> {
        let mut statement = connection.prepare("
            SELECT id
            FROM tags
            WHERE name = $1
        ")?;
        let mut rows = statement.query(&[&tag.to_string()])?;
        let row = rows.next().ok_or_else(|| Error::from_string("tag not found".to_string()))?;
        let id = row.map(|row| row.get(0))?;
        Ok(id)
    }

    /// Get the tags of a bookmark.
    pub fn get_tags(&self, url: &str) -> Result<Vec<String>> {
        CONNECTION.with(|connection| {
            if let Some(ref connection) = *connection.borrow() {
                if let Ok(mut statement) = connection.prepare("
                        SELECT name
                        FROM tags
                        INNER JOIN bookmarks_tags
                            ON tags.id = bookmarks_tags.tag_id
                        INNER JOIN bookmarks
                            ON bookmarks_tags.bookmark_id = bookmarks.id
                        WHERE url = $1
                    ")
                {
                    if let Ok(rows) = statement.query_map(&[&url.to_string()], |row| {
                            row.get(0)
                        })
                    {
                        return rows.collect::<result::Result<Vec<_>, _>>()
                            .map_err(Into::into);
                    }
                }
            }
            Ok(vec![])
        })
    }

    /// Query the bookmarks.
    pub fn query(&self, input: BookmarkInput) -> Vec<Bookmark> {
        CONNECTION.with(|connection| {
            if let Some(ref connection) = *connection.borrow() {
                let mut params: Vec<&ToSql> = vec![];

                let mut title_idents = vec![];
                for title in &input.words {
                    let index = params.len();
                    title_idents.push(format!("(title LIKE '%' || ${} || '%' OR url LIKE '%' || ${} || '%')", index, index + 1));
                    params.push(title);
                    params.push(title);
                }
                let title_idents = title_idents.join(" AND ");
                let where_clause =
                    if !title_idents.is_empty() {
                        format!("WHERE {}", title_idents)
                    }
                    else {
                        String::new()
                    };

                let delta = params.len();
                let mut tag_idents = vec![];
                for (index, tag) in input.tags.iter().enumerate() {
                    tag_idents.push(format!("tags.name LIKE ${} || '%'", index + delta));
                    params.push(tag);
                }
                let tag_idents = tag_idents.join(" OR ");
                let having_clause =
                    if !tag_idents.is_empty() {
                        format!("HAVING COUNT(CASE WHEN {} THEN 1 END) = {}", tag_idents, input.tags.len())
                    }
                    else {
                        String::new()
                    };

                if let Ok(mut statement) = connection.prepare(&format!("
                            SELECT title, url, COALESCE(GROUP_CONCAT(tags.name, ' #'), '')
                            FROM bookmarks
                            LEFT OUTER JOIN bookmarks_tags
                                ON bookmarks.id = bookmarks_tags.bookmark_id
                            LEFT OUTER JOIN tags
                                ON bookmarks_tags.tag_id = tags.id
                            {}
                            GROUP BY url
                            {}
                        ", where_clause, having_clause))
                {
                    if let Ok(rows) = statement.query_map(&params, |row| {
                        Bookmark::new(row.get(1), row.get(0), row.get(2))
                    })
                    {
                        return rows.collect::<result::Result<Vec<_>, _>>().unwrap_or_else(|_| vec![]);
                    }
                }
            }
            vec![]
        })
    }

    /// Set the tags of a bookmark.
    pub fn set_tags(&self, url: &str, tags: Vec<String>) -> Result<()> {
        let original_tags = self.get_tags(url)?;
        CONNECTION.with(|connection| {
            if let Some(bookmark_id) = self.get_id(url) {
                if let Some(ref connection) = *connection.borrow() {
                    for tag in &tags {
                        let tag = tag.to_lowercase();
                        connection.execute("
                            INSERT OR IGNORE INTO tags (name)
                            VALUES ($1)
                        ", &[&tag])?;
                        let tag_id = self.get_tag_id(connection, &tag)?;
                        connection.execute("
                            INSERT OR IGNORE INTO bookmarks_tags (bookmark_id, tag_id)
                            VALUES ($1, $2)
                        ", &[&bookmark_id, &tag_id])
                            .map(|_| ())?
                    }
                    self.delete_tags(connection, bookmark_id, &original_tags, &tags)?;
                }
            }
            Ok(())
        })
    }
}

/// A bookmark input query.
pub struct BookmarkInput {
    pub tags: Vec<String>,
    pub words: Vec<String>,
}
