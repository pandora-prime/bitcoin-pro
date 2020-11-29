// Bitcoin Pro: Professional bitcoin accounts & assets management
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the AGPL License
// along with this software.
// If not, see <https://www.gnu.org/licenses/agpl-3.0-standalone.html>.

#![allow(dead_code)]

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate amplify_derive;
#[macro_use]
extern crate lnpbp;
#[macro_use]
extern crate lnpbp_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate glade;
#[macro_use]
extern crate glib;
#[macro_use]
extern crate serde_with;

mod controller;
mod model;
mod util;
mod view;

use gio::prelude::*;
use std::path::PathBuf;

use crate::view::OpenDlg;

fn main() {
    let application =
        gtk::Application::new(Some("com.pandoracore.BitcoinPro"), default!())
            .expect("BitcoinPro failed to initialize GTK environment");

    application.connect_activate(|_| {
        fn new_app(path: Option<PathBuf>) {
            if let Ok(app_window) = view::AppWindow::new(path) {
                let app_window = app_window.borrow();
                app_window.run(
                    || {
                        let open_dlg =
                            OpenDlg::load_glade().expect("Must load");
                        open_dlg.run(move |path| new_app(Some(path)), || {})
                    },
                    || {
                        new_app(None);
                    },
                );
            }
        }

        new_app(None);
    });

    application.run(&std::env::args().collect::<Vec<_>>());
}
