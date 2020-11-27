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

use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::model::Document;

static UI: &'static str = include_str!("../../ui/pubkey_select.glade");

#[derive(Debug, Display, From, Error)]
#[display(doc_comments)]
/// Errors from processing descriptor data
pub enum Error {
    /// Temporary error
    None,
}

pub struct PubkeySelectDlg {
    dialog: gtk::Dialog,
    pubkey_store: gtk::ListStore,
    pubkey_selection: gtk::TreeSelection,
    select_btn: gtk::Button,
    cancel_btn: gtk::Button,
}

impl PubkeySelectDlg {
    pub fn load_glade() -> Result<Rc<Self>, glade::Error> {
        let builder = gtk::Builder::from_string(UI);

        let pubkey_store = builder.get_object("pubkeyStore")?;
        let pubkey_selection = builder.get_object("pubkeySelection")?;

        let select_btn = builder.get_object("select")?;
        let cancel_btn = builder.get_object("cancel")?;

        let me = Rc::new(Self {
            dialog: glade_load!(builder, "pubkeyDlg")?,
            pubkey_store,
            pubkey_selection,
            select_btn,
            cancel_btn,
        });

        Ok(me)
    }
}

impl PubkeySelectDlg {
    pub fn run(
        self: Rc<Self>,
        doc: Rc<RefCell<Document>>,
        on_select: impl Fn(String) + 'static,
        on_cancel: impl Fn() + 'static,
    ) {
        let me = self.clone();

        doc.borrow().fill_tracking_store(&me.pubkey_store);

        me.cancel_btn
            .connect_clicked(clone!(@weak self as me => move |_| {
                me.dialog.hide();
                on_cancel()
            }));

        me.select_btn.connect_clicked(
            clone!(@weak self as me => move |_| match self.selected_pubkey() {
                Some(selected_pubkey) => {
                    me.dialog.hide();
                    on_select(selected_pubkey);
                }
                None => {
                    me.select_btn.set_sensitive(false);
                }
            }),
        );

        me.dialog.run();
        me.dialog.hide();
    }

    pub fn selected_pubkey(&self) -> Option<String> {
        self.pubkey_selection.get_selected().map(|(model, iter)| {
            model
                .get_value(&iter, 1)
                .get::<String>()
                .expect("Pubkey selection not found")
                .expect("Pubkey kye string not found")
        })
    }
}
