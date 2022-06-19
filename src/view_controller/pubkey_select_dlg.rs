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

use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use gtk::ResponseType;

use crate::model::{Document, TrackingAccount};

static UI: &str = include_str!("../view/pubkey_select.glade");

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
    pub fn load_glade() -> Option<Rc<Self>> {
        let builder = gtk::Builder::from_string(UI);

        let pubkey_store = builder.object("pubkeyStore")?;
        let pubkey_selection = builder.object("pubkeySelection")?;

        let select_btn = builder.object("select")?;
        let cancel_btn = builder.object("cancel")?;

        let me = Rc::new(Self {
            dialog: glade_load!(builder, "pubkeyDlg").ok()?,
            pubkey_store,
            pubkey_selection,
            select_btn,
            cancel_btn,
        });

        Some(me)
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
                match me
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
            .set_sensitive(self.pubkey_selection.selected().is_some())
    }

    pub fn selected_pubkey(&self) -> Option<String> {
        self.pubkey_selection.selected().map(|(model, iter)| {
            model
                .value(&iter, 1)
                .get::<String>()
                .expect("Pubkey selection not found")
        })
    }
}
