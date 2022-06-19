// Bitcoin Pro: Professional bitcoin accounts & assets management
// Written in 2020-2022 by
//     Dr. Maxim Orlovsky <orlovsky@pandoraprime.ch>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![allow(dead_code)]
// TODO: Remove once bugs in amplify_derive and strict_encode are fixed
#![allow(clippy::if_same_then_else, clippy::init_numbered_fields)]

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate amplify_derive;
#[macro_use]
extern crate lnpbp;
#[macro_use]
extern crate glib;

#[macro_export]
macro_rules! glade_load {
    ($builder:ident, $file:literal) => {
        $builder.object($file).ok_or($crate::Error::ParseFailed)
    };
}

#[derive(Clone, PartialEq, Eq, Debug, Display, From, Error)]
#[display(doc_comments)]
pub enum Error {
    /// Failed to parse glade file
    ParseFailed,

    /// The specified widget is not found
    WidgetNotFound,
}

pub trait View
where
    Self: Sized,
{
    fn load_glade() -> Result<Rc<RefCell<Self>>, Error>;
}

mod controller;
mod model;
mod util;
mod view_controller;

use gtk::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::view_controller::OpenDlg;

fn main() {
    let application =
        gtk::Application::new(Some("com.pandoracore.BitcoinPro"), default!());

    application.connect_activate(|_| {
        fn new_app(path: Option<PathBuf>) {
            if let Some(app_window) = view_controller::BproWin::new(path) {
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

    application.run();
}
