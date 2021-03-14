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

use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use gtk::ResponseType;

use crate::model::{Document, TrackingAccount};

static UI: &'static str = include_str!("../view/pubkey_select.glade");

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
        on_select: impl Fn(TrackingAccount) + 'static,
        on_cancel: impl Fn() + 'static,
    ) {
        doc.borrow().fill_tracking_store(&self.pubkey_store);

        self.pubkey_selection.connect_changed(
            clone!(@weak self as me => move |_| {
                me.update_ui();
            }),
        );

        self.cancel_btn
            .connect_clicked(clone!(@weak self as me => move |_| {
                me.dialog.response(ResponseType::Cancel);
                on_cancel();
            }));

        self.select_btn
            .connect_clicked(clone!(@weak self as me => move |_| {
                match me.clone()
                        .selected_pubkey()
                        .and_then(|k| doc.borrow().tracking_account_by_key(&k)) {
                    Some(tracking_account) => {
                        me.dialog.response(ResponseType::Ok);
                        on_select(tracking_account);
                    }
                    None => {
                        me.select_btn.set_sensitive(false);
                    }
                }
            }));

        self.update_ui();

        self.dialog.run();
        self.dialog.hide();
    }

    pub fn update_ui(&self) {
        self.select_btn
            .set_sensitive(self.pubkey_selection.get_selected().is_some())
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
