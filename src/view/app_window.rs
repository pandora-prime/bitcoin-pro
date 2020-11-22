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

use glade::View;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use super::PubkeyDlg;

static UI: &'static str = include_str!("../../ui/main.glade");

#[derive(Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum Error {
    /// Glade error: {0}
    #[from]
    GladeError(glade::Error),
}

pub struct AppWindow {
    window: gtk::ApplicationWindow,
}

impl View for AppWindow {
    fn load_glade() -> Result<Rc<RefCell<Self>>, glade::Error> {
        let builder = gtk::Builder::from_string(UI);

        let tb: gtk::ToolButton = builder
            .get_object("pubkeyAdd")
            .ok_or(glade::Error::WidgetNotFound)?;
        let pubkey_dlg = PubkeyDlg::load_glade()?;
        tb.connect_clicked(move |_| {
            pubkey_dlg.borrow().run();
        });

        Ok(Rc::new(RefCell::new(Self {
            window: glade_load!(builder, "appWindow")?,
        })))
    }
}

impl AppWindow {
    pub fn new() -> Result<Rc<RefCell<Self>>, Error> {
        let me = Self::load_glade()?;
        Ok(me)
    }

    pub fn run(&mut self) {
        self.update();
        self.window.show_all();
        gtk::main();
    }

    pub fn update(&mut self) {}
}
