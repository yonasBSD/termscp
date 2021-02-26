//! ## AuthActivity
//!
//! `auth_activity` is the module which implements the authentication activity

/*
*
*   Copyright (C) 2020-2021 Christian Visintin - christian.visintin1997@gmail.com
*
* 	This file is part of "TermSCP"
*
*   TermSCP is free software: you can redistribute it and/or modify
*   it under the terms of the GNU General Public License as published by
*   the Free Software Foundation, either version 3 of the License, or
*   (at your option) any later version.
*
*   TermSCP is distributed in the hope that it will be useful,
*   but WITHOUT ANY WARRANTY; without even the implied warranty of
*   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
*   GNU General Public License for more details.
*
*   You should have received a copy of the GNU General Public License
*   along with TermSCP.  If not, see <http://www.gnu.org/licenses/>.
*
*/

// Dependencies
extern crate dirs;

// Locals
use super::{AuthActivity, Color, DialogYesNoOption, Popup};
use crate::system::bookmarks_client::BookmarksClient;
use crate::system::environment;

// Ext
use std::path::PathBuf;

impl AuthActivity {
    /// ### del_bookmark
    ///
    /// Delete bookmark
    pub(super) fn del_bookmark(&mut self, idx: usize) {
        if let Some(bookmarks_cli) = self.bookmarks_client.as_mut() {
            // Iterate over kyes
            let name: Option<&String> = self.bookmarks_list.get(idx);
            if let Some(name) = name {
                bookmarks_cli.del_bookmark(&name);
                // Write bookmarks
                self.write_bookmarks();
            }
            // Delete element from vec
            self.recents_list.remove(idx);
        }
    }

    /// ### load_bookmark
    ///
    /// Load selected bookmark (at index) to input fields
    pub(super) fn load_bookmark(&mut self, idx: usize) {
        if let Some(bookmarks_cli) = self.bookmarks_client.as_ref() {
            // Iterate over bookmarks
            if let Some(key) = self.bookmarks_list.get(idx) {
                if let Some(bookmark) = bookmarks_cli.get_bookmark(&key) {
                    // Load parameters
                    self.address = bookmark.0;
                    self.port = bookmark.1.to_string();
                    self.protocol = bookmark.2;
                    self.username = bookmark.3;
                    if let Some(password) = bookmark.4 {
                        self.password = password;
                    }
                }
            }
        }
    }

    /// ### save_bookmark
    ///
    /// Save current input fields as a bookmark
    pub(super) fn save_bookmark(&mut self, name: String) {
        // Check port
        let port: u16 = match self.port.parse::<usize>() {
            Ok(val) => {
                if val > 65535 {
                    self.popup = Some(Popup::Alert(
                        Color::Red,
                        String::from("Specified port must be in range 0-65535"),
                    ));
                    return;
                }
                val as u16
            }
            Err(_) => {
                self.popup = Some(Popup::Alert(
                    Color::Red,
                    String::from("Specified port is not a number"),
                ));
                return;
            }
        };
        if let Some(bookmarks_cli) = self.bookmarks_client.as_mut() {
            // Check if password must be saved
            let password: Option<String> = match self.choice_opt {
                DialogYesNoOption::Yes => Some(self.password.clone()),
                DialogYesNoOption::No => None,
            };
            bookmarks_cli.add_bookmark(
                name.clone(),
                self.address.clone(),
                port,
                self.protocol,
                self.username.clone(),
                password,
            );
            // Save bookmarks
            self.write_bookmarks();
            // Push bookmark to list and sort
            self.bookmarks_list.push(name);
            self.sort_bookmarks();
        }
    }
    /// ### del_recent
    ///
    /// Delete recent
    pub(super) fn del_recent(&mut self, idx: usize) {
        if let Some(client) = self.bookmarks_client.as_mut() {
            let name: Option<&String> = self.recents_list.get(idx);
            if let Some(name) = name {
                client.del_recent(&name);
                // Write bookmarks
                self.write_bookmarks();
            }
            // Delete element from vec
            self.recents_list.remove(idx);
        }
    }

    /// ### load_recent
    ///
    /// Load selected recent (at index) to input fields
    pub(super) fn load_recent(&mut self, idx: usize) {
        if let Some(client) = self.bookmarks_client.as_ref() {
            // Iterate over bookmarks
            if let Some(key) = self.recents_list.get(idx) {
                if let Some(bookmark) = client.get_recent(key) {
                    // Load parameters
                    self.address = bookmark.0;
                    self.port = bookmark.1.to_string();
                    self.protocol = bookmark.2;
                    self.username = bookmark.3;
                }
            }
        }
    }

    /// ### save_recent
    ///
    /// Save current input fields as a "recent"
    pub(super) fn save_recent(&mut self) {
        // Check port
        let port: u16 = match self.port.parse::<usize>() {
            Ok(val) => {
                if val > 65535 {
                    self.popup = Some(Popup::Alert(
                        Color::Red,
                        String::from("Specified port must be in range 0-65535"),
                    ));
                    return;
                }
                val as u16
            }
            Err(_) => {
                self.popup = Some(Popup::Alert(
                    Color::Red,
                    String::from("Specified port is not a number"),
                ));
                return;
            }
        };
        if let Some(bookmarks_cli) = self.bookmarks_client.as_mut() {
            bookmarks_cli.add_recent(
                self.address.clone(),
                port,
                self.protocol,
                self.username.clone(),
            );
            // Save bookmarks
            self.write_bookmarks();
        }
    }

    /// ### write_bookmarks
    ///
    /// Write bookmarks to file
    fn write_bookmarks(&mut self) {
        if let Some(bookmarks_cli) = self.bookmarks_client.as_ref() {
            if let Err(err) = bookmarks_cli.write_bookmarks() {
                self.popup = Some(Popup::Alert(
                    Color::Red,
                    format!("Could not write bookmarks: {}", err),
                ));
            }
        }
    }

    /// ### init_bookmarks_client
    ///
    /// Initialize bookmarks client
    pub(super) fn init_bookmarks_client(&mut self) {
        // Get config dir
        match environment::init_config_dir() {
            Ok(path) => {
                // If some configure client, otherwise do nothing; don't bother users telling them that bookmarks are not supported on their system.
                if let Some(config_dir_path) = path {
                    let bookmarks_file: PathBuf =
                        environment::get_bookmarks_paths(config_dir_path.as_path());
                    // Initialize client
                    match BookmarksClient::new(
                        bookmarks_file.as_path(),
                        config_dir_path.as_path(),
                        16,
                    ) {
                        Ok(cli) => {
                            // Load bookmarks into list
                            let mut bookmarks_list: Vec<String> =
                                Vec::with_capacity(cli.iter_bookmarks().count());
                            for bookmark in cli.iter_bookmarks() {
                                bookmarks_list.push(bookmark.clone());
                            }
                            // Load recents into list
                            let mut recents_list: Vec<String> =
                                Vec::with_capacity(cli.iter_recents().count());
                            for recent in cli.iter_recents() {
                                recents_list.push(recent.clone());
                            }
                            self.bookmarks_client = Some(cli);
                            self.bookmarks_list = bookmarks_list;
                            self.recents_list = recents_list;
                            // Sort bookmark list
                            self.sort_bookmarks();
                            self.sort_recents();
                        }
                        Err(err) => {
                            self.popup = Some(Popup::Alert(
                                Color::Red,
                                format!(
                                    "Could not initialize bookmarks (at \"{}\", \"{}\"): {}",
                                    bookmarks_file.display(),
                                    config_dir_path.display(),
                                    err
                                ),
                            ))
                        }
                    }
                }
            }
            Err(err) => {
                self.popup = Some(Popup::Alert(
                    Color::Red,
                    format!("Could not initialize configuration directory: {}", err),
                ))
            }
        }
    }

    /// ### sort_bookmarks
    ///
    /// Sort bookmarks in list
    fn sort_bookmarks(&mut self) {
        // Conver to lowercase when sorting
        self.bookmarks_list
            .sort_by(|a, b| a.to_lowercase().as_str().cmp(b.to_lowercase().as_str()));
    }

    /// ### sort_recents
    ///
    /// Sort recents in list
    fn sort_recents(&mut self) {
        // Reverse order
        self.recents_list.sort_by(|a, b| b.cmp(a));
    }
}
