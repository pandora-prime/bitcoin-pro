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
use gtk::ResponseType;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use bitcoin::{OutPoint, Txid};

use crate::model::{Document, UtxoEntry};

static UI: &str = include_str!("../view/utxo_select.glade");

#[derive(Debug, Display, From, Error)]
#[display(doc_comments)]
/// Errors from processing descriptor data
pub enum Error {
    /// Temporary error
    None,
}

pub struct UtxoSelectDlg {
    dialog: gtk::Dialog,
    descriptor_store: gtk::ListStore,
    descriptor_selection: gtk::TreeSelection,
    utxo_store: gtk::ListStore,
    utxo_selection: gtk::TreeSelection,
    select_btn: gtk::Button,
    cancel_btn: gtk::Button,
}

impl UtxoSelectDlg {
    pub fn load_glade() -> Option<Rc<Self>> {
        let builder = gtk::Builder::from_string(UI);

        let descriptor_store = builder.get_object("locatorStore")?;
        let descriptor_selection = builder.get_object("locatorSelection")?;
        let utxo_store = builder.get_object("utxoStore")?;
        let utxo_selection = builder.get_object("utxoSelection")?;

        let select_btn = builder.get_object("select")?;
        let cancel_btn = builder.get_object("cancel")?;

        let me = Rc::new(Self {
            dialog: glade_load!(builder, "utxoDlg").ok()?,
            descriptor_store,
            descriptor_selection,
            utxo_store,
            utxo_selection,
            select_btn,
            cancel_btn,
        });

        Some(me)
    }
}

impl UtxoSelectDlg {
    pub fn run(
        self: Rc<Self>,
        doc: Rc<RefCell<Document>>,
        on_select: impl Fn(UtxoEntry) + 'static,
        on_cancel: impl Fn() + 'static,
    ) {
        doc.borrow().fill_descriptor_store(&self.descriptor_store);

        self.descriptor_selection.connect_changed(
            clone!(@weak self as me, @strong doc => move |_| {
                me.utxo_store.clear();
                if let Some(descriptor_generator) =
                    me.descriptor_selection().and_then(|(generator, _, _)| {
                        doc.borrow().descriptor_by_generator(&generator)
                    })
                {
                    doc.borrow().fill_utxo_store(&me.utxo_store, Some(&descriptor_generator));
                }
                me.update_ui();
            }),
        );

        self.utxo_selection.connect_changed(
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
                        .selected_outpoint()
                        .and_then(|outpoint| {
                            doc.borrow().utxo_by_outpoint(outpoint)
                        }) {
                    Some(utxo) => {
                        me.dialog.response(ResponseType::Ok);
                        on_select(utxo);
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
            .set_sensitive(self.utxo_selection.get_selected().is_some());
    }

    pub fn descriptor_selection(
        &self,
    ) -> Option<(String, gtk::TreeModel, gtk::TreeIter)> {
        self.descriptor_selection
            .get_selected()
            .and_then(|(model, iter)| {
                model
                    .get_value(&iter, 2)
                    .get::<String>()
                    .ok()
                    .flatten()
                    .map(|name| (name, model, iter))
            })
    }

    pub fn selected_outpoint(&self) -> Option<OutPoint> {
        self.utxo_selection
            .get_selected()
            .and_then(|(model, iter)| {
                let txid = model
                    .get_value(&iter, 0)
                    .get::<String>()
                    .ok()
                    .flatten()
                    .map(|txid| Txid::from_str(&txid))
                    .transpose()
                    .ok()
                    .flatten();
                let vout =
                    model.get_value(&iter, 1).get::<u32>().ok().flatten();
                vout.and_then(|vout| txid.map(|txid| OutPoint { txid, vout }))
            })
    }
}
