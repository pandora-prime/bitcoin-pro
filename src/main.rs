// Bitcoin Pro: Professional bitcoin accounts & assets management
// Written in 2020-2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

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
extern crate lazy_static;
#[macro_use]
extern crate glib;

#[macro_export]
macro_rules! glade_load {
    ($builder:ident, $file:literal) => {
        $builder.get_object($file).ok_or($crate::Error::ParseFailed)
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

use gio::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::view_controller::OpenDlg;

fn main() {
    let application =
        gtk::Application::new(Some("com.pandoracore.BitcoinPro"), default!())
            .expect("BitcoinPro failed to initialize GTK environment");

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

    application.run(&std::env::args().collect::<Vec<_>>());
}
