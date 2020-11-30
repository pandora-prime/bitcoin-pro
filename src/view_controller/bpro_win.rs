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
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;

use lnpbp::bitcoin::{OutPoint, Txid};

use crate::model::Document;
use crate::view_controller::{AssetDlg, DescriptorDlg, PubkeyDlg, SaveDlg};

static UI: &'static str = include_str!("../view/bpro.glade");

#[derive(Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum Error {
    /// Glade error: {0}
    #[from]
    GladeError(glade::Error),

    /// Document-based error
    #[from]
    #[display("{0}")]
    Document(crate::model::Error),
}

pub struct BproWin {
    window: gtk::ApplicationWindow,
    pubkey_tree: gtk::TreeView,
    pubkey_store: gtk::ListStore,
    descriptor_tree: gtk::TreeView,
    descriptor_store: gtk::ListStore,
    utxo_descr_tree: gtk::TreeView,
    utxo_descr_store: gtk::ListStore,
    utxo_tree: gtk::TreeView,
    utxo_store: gtk::ListStore,
    header_bar: gtk::HeaderBar,
    new_btn: gtk::Button,
    open_btn: gtk::Button,
    pubkey_edit_btn: gtk::ToolButton,
    pubkey_remove_btn: gtk::ToolButton,
    descriptor_edit_btn: gtk::ToolButton,
    descriptor_remove_btn: gtk::ToolButton,
    utxo_descr_remove_btn: gtk::ToolButton,
    utxo_descr_clear_btn: gtk::ToolButton,
    utxo_remove_btn: gtk::ToolButton,
}

impl BproWin {
    fn load_glade(
        doc: Option<Document>,
    ) -> Result<Rc<RefCell<Self>>, glade::Error> {
        let mut needs_save = true;
        let doc = Rc::new(RefCell::new(if let Some(doc) = doc {
            needs_save = false;
            doc
        } else {
            Document::new()
        }));

        let builder = gtk::Builder::from_string(UI);

        let new_btn: gtk::Button = builder.get_object("new")?;
        let open_btn: gtk::Button = builder.get_object("open")?;
        let header_bar: gtk::HeaderBar = builder.get_object("headerBar")?;

        let pubkey_edit_btn = builder.get_object("pubkeyEdit")?;
        let pubkey_remove_btn = builder.get_object("pubkeyRemove")?;
        let descriptor_edit_btn = builder.get_object("descriptorEdit")?;
        let descriptor_remove_btn = builder.get_object("descriptorRemove")?;
        let utxo_descr_remove_btn = builder.get_object("utxoDescrRemove")?;
        let utxo_descr_clear_btn = builder.get_object("utxoDescrClear")?;
        let utxo_remove_btn = builder.get_object("utxoRemove")?;

        let pubkey_tree: gtk::TreeView = builder.get_object("pubkeyTree")?;
        let pubkey_store = builder.get_object("pubkeyStore")?;
        let descriptor_tree: gtk::TreeView =
            builder.get_object("locatorTree")?;
        let descriptor_store = builder.get_object("locatorStore")?;
        let utxo_descr_tree: gtk::TreeView =
            builder.get_object("utxoDescrTree")?;
        let utxo_descr_store = builder.get_object("utxoDescrStore")?;
        let utxo_tree: gtk::TreeView = builder.get_object("utxoTree")?;
        let utxo_store = builder.get_object("utxoStore")?;

        let chain_combo: gtk::ComboBox = builder.get_object("chainCombo")?;
        let electrum_radio: gtk::RadioButton =
            builder.get_object("electrum")?;
        let electrum_field: gtk::Entry = builder.get_object("electrumField")?;
        let electrum_btn: gtk::Button = builder.get_object("electrumBtn")?;

        doc.borrow().fill_tracking_store(&pubkey_store);
        doc.borrow().fill_descriptor_store(&descriptor_store);
        doc.borrow().fill_utxo_store(&utxo_store, None);

        header_bar.set_subtitle(Some(&doc.borrow().name()));

        chain_combo.set_active_id(Some(&doc.borrow().chain().to_string()));
        electrum_radio.set_active(true);
        electrum_field.set_text(&doc.borrow().electrum().unwrap_or_default());

        let me = Rc::new(RefCell::new(Self {
            window: glade_load!(builder, "appWindow")?,
            pubkey_tree,
            pubkey_store,
            descriptor_tree,
            descriptor_store,
            utxo_descr_tree,
            utxo_descr_store,
            utxo_tree,
            utxo_store,
            header_bar,
            new_btn,
            open_btn,
            pubkey_edit_btn,
            pubkey_remove_btn,
            descriptor_edit_btn,
            descriptor_remove_btn,
            utxo_descr_remove_btn,
            utxo_descr_clear_btn,
            utxo_remove_btn,
        }));

        chain_combo.connect_changed(
            clone!(@weak chain_combo, @strong doc => move |_| {
                if let Some(chain_name) = chain_combo.get_active_id() {
                    let _ = doc.borrow_mut().set_chain(&chain_name);
                }
            }),
        );

        electrum_field.connect_changed(
            clone!(@strong doc, @weak electrum_field => move |_| {
                match electrum_field.get_text().to_string().parse() {
                    Ok(addr) => {
                        electrum_field.set_property_secondary_icon_name(None);
                        electrum_field.set_property_secondary_icon_tooltip_text(
                            Some("")
                        );
                        let _ = doc.borrow_mut().set_electrum(addr);
                    }
                    Err(err) => {
                        electrum_field.set_property_secondary_icon_name(
                            Some("dialog-error")
                        );
                        electrum_field.set_property_secondary_icon_tooltip_text(
                            Some(&err.to_string())
                        );
                    }
                }
            }),
        );

        electrum_btn.connect_clicked(
            clone!(@strong doc, @weak electrum_field => move |_| {
                if let Err(err) = doc.borrow().resolver() {
                    electrum_field.set_property_secondary_icon_name(
                        Some("dialog-error")
                    );
                    electrum_field.set_property_secondary_icon_tooltip_text(
                        Some(&err.to_string())
                    );
                } else {
                    electrum_field.set_property_secondary_icon_name(
                        Some("dialog-ok")
                    );
                    electrum_field.set_property_secondary_icon_tooltip_text(
                        Some("")
                    );
                }
            }),
        );

        me.borrow().pubkey_tree.get_selection().connect_changed(
            clone!(@weak me, @strong doc => move |_| {
                let me = me.borrow();
                if let Some(_) = me.pubkey_selection() {
                    me.pubkey_edit_btn.set_sensitive(true);
                    me.pubkey_remove_btn.set_sensitive(true);
                } else {
                    me.pubkey_edit_btn.set_sensitive(false);
                    me.pubkey_remove_btn.set_sensitive(false);
                }
            }),
        );

        let tb: gtk::ToolButton = builder.get_object("pubkeyAdd")?;
        tb.connect_clicked(clone!(@weak me, @strong doc => move |_| {
            let pubkey_dlg = PubkeyDlg::load_glade().expect("Must load");
            pubkey_dlg.run(None, doc.borrow().chain(), clone!(@weak me, @strong doc =>
                move |tracking_account| {
                    let me = me.borrow();
                    me.pubkey_store.insert_with_values(
                        None,
                        &[0, 1, 2],
                        &[
                            &tracking_account.name(),
                            &tracking_account.details(),
                            &tracking_account.count(),
                        ],
                    );
                    let _ = doc.borrow_mut().add_tracking_account(tracking_account);
                }),
                || {},
            );
        }));

        me.borrow().pubkey_edit_btn.connect_clicked(clone!(@weak me, @strong doc => move |_| {
            let meb = me.borrow();
            let pubkey_dlg = PubkeyDlg::load_glade().expect("Must load");
            if let Some((keyname, _, iter)) = meb.pubkey_selection() {
                let tracking_account = doc
                    .borrow()
                    .tracking_account_by_key(&keyname)
                    .expect("Tracking account must be known since it is selected");
                pubkey_dlg.run(Some(tracking_account.clone()), doc.borrow().chain(), clone!(@weak me, @strong doc =>
                    move |new_tracking_account| {
                        let me = me.borrow();
                        me.pubkey_store.set_value(&iter, 0, &new_tracking_account.name().to_value());
                        me.pubkey_store.set_value(&iter, 1, &new_tracking_account.details().to_value());
                        me.pubkey_store.set_value(&iter, 2, &new_tracking_account.count().to_value());
                        let _ = doc.borrow_mut().update_tracking_account(&tracking_account, new_tracking_account);
                    }),
                    || {},
                );
            }
        }));

        me.borrow().pubkey_remove_btn.connect_clicked(clone!(@weak me, @strong doc => move |_| {
            let me = me.borrow();
            if let Some((keyname, _, iter)) = me.pubkey_selection() {
                let tracking_account = doc
                    .borrow()
                    .tracking_account_by_key(&keyname)
                    .expect("Tracking account must be known since it is selected");
                let dlg = gtk::MessageDialog::new(
                    Some(&me.window),
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Question,
                    gtk::ButtonsType::YesNo,
                    &format!(
                        "Please confirm deletion of the public key tracking account for {}", 
                        tracking_account.key
                    )
                );
                if dlg.run() == gtk::ResponseType::Yes {
                    me.pubkey_store.remove(&iter);
                    let _ = doc.borrow_mut().remove_tracking_account(tracking_account);
                }
                dlg.hide();
            }
        }));

        me.borrow().descriptor_tree.get_selection().connect_changed(
            clone!(@weak me, @strong doc => move |_| {
                let me = me.borrow();
                me.utxo_descr_store.clear();
                if let Some((generator, _, _)) = me.descriptor_selection() {
                    if let Some(descriptor_generator) = doc.borrow().descriptor_by_generator(&generator) {
                        doc.borrow().fill_utxo_store(&me.utxo_descr_store, Some(&descriptor_generator));
                    }
                    me.descriptor_edit_btn.set_sensitive(true);
                    me.descriptor_remove_btn.set_sensitive(true);
                } else {
                    me.descriptor_edit_btn.set_sensitive(false);
                    me.descriptor_remove_btn.set_sensitive(false);
                }
                me.utxo_descr_clear_btn.set_sensitive(me.utxo_descr_store.get_iter_first().is_some());
            }),
        );

        let tb: gtk::ToolButton = builder.get_object("descriptorAdd")?;
        tb.connect_clicked(clone!(@weak me, @strong doc => move |_| {
            let descriptor_dlg = DescriptorDlg::load_glade().expect("Must load");
            descriptor_dlg.run(doc.clone(), None, clone!(@weak me, @strong doc =>
                move |descriptor_generator, utxo_set_update| {
                    let me = me.borrow();
                    me.descriptor_store.insert_with_values(
                        None,
                        &[0, 1, 2],
                        &[
                            &descriptor_generator.name(),
                            &descriptor_generator.type_name(),
                            &descriptor_generator.descriptor(),
                        ],
                    );
                    let _ = doc.borrow_mut().add_descriptor(descriptor_generator);
                    let _ = doc.borrow_mut().update_utxo_set(utxo_set_update);
                }),
                || {},
            );
        }));

        me.borrow().descriptor_edit_btn.connect_clicked(clone!(@weak me, @strong doc => move |_| {
            let meb = me.borrow();
            let descriptor_dlg = DescriptorDlg::load_glade().expect("Must load");
            if let Some((generator, _, iter)) = meb.descriptor_selection() {
                let descriptor_generator = doc
                    .borrow()
                    .descriptor_by_generator(&generator)
                    .expect("Descriptor account must be known since it is selected");
                descriptor_dlg.run(doc.clone(), Some(descriptor_generator.clone()), clone!(@weak me, @strong doc =>
                    move |new_descriptor_generator, utxo_set_update| {
                        let me = me.borrow();
                        me.utxo_descr_clear_btn.set_sensitive(!utxo_set_update.is_empty());
                        me.descriptor_store.set_value(&iter, 0, &new_descriptor_generator.name().to_value());
                        me.descriptor_store.set_value(&iter, 1, &new_descriptor_generator.type_name().to_value());
                        me.descriptor_store.set_value(&iter, 2, &new_descriptor_generator.descriptor().to_value());
                        let _ = doc.borrow_mut().update_descriptor(&descriptor_generator, new_descriptor_generator);
                        let _ = doc.borrow_mut().update_utxo_set(utxo_set_update);
                        doc.borrow().fill_utxo_store(&me.utxo_descr_store, Some(&descriptor_generator));
                        doc.borrow().fill_utxo_store(&me.utxo_store, None);
                    }),
                    || {},
                );
            }
        }));

        me.borrow().descriptor_remove_btn.connect_clicked(clone!(@weak me, @strong doc => move |_| {
            let me = me.borrow();
            if let Some((generator, _, iter)) = me.descriptor_selection() {
                let descriptor_generator = doc
                    .borrow()
                    .descriptor_by_generator(&generator)
                    .expect("Descriptor must be known since it is selected");
                let dlg = gtk::MessageDialog::new(
                    Some(&me.window),
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Question,
                    gtk::ButtonsType::YesNo,
                    &format!(
                        "Please confirm deletion of the descriptor '{}' defined by {}",
                        descriptor_generator.name(),
                        descriptor_generator.descriptor()
                    )
                );
                if dlg.run() == gtk::ResponseType::Yes {
                    me.descriptor_store.remove(&iter);
                    let _ = doc.borrow_mut().remove_descriptor(descriptor_generator);
                }
                dlg.hide();
            }
        }));

        me.borrow().utxo_descr_tree.get_selection().connect_changed(
            clone!(@weak me, @strong doc => move |_| {
                let me = me.borrow();
                me.utxo_descr_remove_btn.set_sensitive(me.utxo_descr_tree.get_selection().get_selected().is_some());
            }),
        );

        me.borrow().utxo_descr_remove_btn.connect_clicked(clone!(@weak me, @strong doc => move |_| {
            let me = me.borrow();
            if let Some((outpoint, _, iter)) = Self::utxo_selection(&me.utxo_descr_tree) {
                let utxo = doc
                    .borrow()
                    .utxo_by_outpoint(outpoint)
                    .expect("UTXO must be known since it is selected");
                let dlg = gtk::MessageDialog::new(
                    Some(&me.window),
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Question,
                    gtk::ButtonsType::YesNo,
                    &format!("Please confirm deletion of {}", utxo)
                );
                if dlg.run() == gtk::ResponseType::Yes {
                    me.utxo_descr_store.remove(&iter);
                    let _ = doc.borrow_mut().remove_utxo(utxo);
                    doc.borrow().fill_utxo_store(&me.utxo_store, None);
                }
                dlg.hide();
            }
        }));

        me.borrow().utxo_descr_clear_btn.connect_clicked(clone!(@weak me, @strong doc => move |_| {
            let me = me.borrow();
            if let Some((generator, _, _)) = me.descriptor_selection() {
                let descriptor_generator = doc
                    .borrow()
                    .descriptor_by_generator(&generator)
                    .expect("Descriptor must be known since it is selected");
                let dlg = gtk::MessageDialog::new(
                    Some(&me.window),
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Question,
                    gtk::ButtonsType::YesNo,
                    &format!("Please confirm deletion of all UTXOs for {}", generator)
                );
                if dlg.run() == gtk::ResponseType::Yes {
                    me.utxo_descr_store.clear();
                    let _ = doc.borrow_mut().remove_utxo_by_descriptor(descriptor_generator);
                    doc.borrow().fill_utxo_store(&me.utxo_store, None);
                    me.utxo_descr_clear_btn.set_sensitive(false);
                }
                dlg.hide();
            }
        }));

        me.borrow().utxo_tree.get_selection().connect_changed(
            clone!(@weak me, @strong doc => move |_| {
                let me = me.borrow();
                me.utxo_remove_btn.set_sensitive(me.utxo_tree.get_selection().get_selected().is_some());
            }),
        );

        me.borrow().utxo_remove_btn.connect_clicked(clone!(@weak me, @strong doc => move |_| {
            let me = me.borrow();
            if let Some((outpoint, _, iter)) = Self::utxo_selection(&me.utxo_tree) {
                let utxo = doc
                    .borrow()
                    .utxo_by_outpoint(outpoint)
                    .expect("UTXO must be known since it is selected");
                let dlg = gtk::MessageDialog::new(
                    Some(&me.window),
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Question,
                    gtk::ButtonsType::YesNo,
                    &format!("Please confirm deletion of {}", utxo)
                );
                if dlg.run() == gtk::ResponseType::Yes {
                    me.utxo_store.remove(&iter);
                    let _ = doc.borrow_mut().remove_utxo(utxo);
                    if let Some((generator, _, _)) = me.descriptor_selection() {
                        let descriptor_generator = doc
                            .borrow()
                            .descriptor_by_generator(&generator)
                            .expect("Descriptor must be known since it is selected");
                        doc.borrow().fill_utxo_store(&me.utxo_descr_store, Some(&descriptor_generator));
                    } else {
                        me.utxo_descr_store.clear();
                    }
                    me.utxo_descr_clear_btn.set_sensitive(me.utxo_descr_store.get_iter_first().is_some());
                }
                dlg.hide();
            }
        }));

        let tb: gtk::ToolButton = builder.get_object("assetCreate")?;
        tb.connect_clicked(clone!(@weak me, @strong doc => move |_| {
            let issue_dlg = AssetDlg::load_glade().expect("Must load");
            issue_dlg.run(doc.clone(), None, clone!(@weak me, @strong doc =>
                move |_asset_genesis| {
                    /* TODO: Perform assst creation
                    let me = me.borrow();
                    me.pubkey_store.insert_with_values(
                        None,
                        &[0, 1, 2],
                        &[
                            &tracking_account.name(),
                            &tracking_account.details(),
                            &tracking_account.count(),
                        ],
                    );
                    let _ = doc.borrow_mut().add_tracking_account(tracking_account);
                     */
                }),
                || {},
            );
        }));

        let tb: gtk::Button = builder.get_object("save")?;
        tb.set_sensitive(needs_save);
        tb.connect_clicked(clone!(@strong doc, @weak tb => move |_| {
            let save_dlg = SaveDlg::load_glade().expect("Must load");
            let name = doc.borrow().name();
            save_dlg.run(name, clone!(@strong doc, @weak tb => move |path| {
                let mut path = path;
                path.set_extension("bpro");
                let _ = doc.borrow_mut().save_as(path).and_then(|_| {
                    tb.set_sensitive(false);
                    Ok(())
                });
            }), || {})
        }));

        Ok(me)
    }
}

impl BproWin {
    pub fn new(path: Option<PathBuf>) -> Result<Rc<RefCell<Self>>, Error> {
        let doc = if let Some(path) = path {
            Some(Document::load(path)?)
        } else {
            None
        };
        let me = Self::load_glade(doc)?;
        Ok(me)
    }

    pub fn run(
        &self,
        on_open: impl Fn() + 'static,
        on_new: impl Fn() + 'static,
    ) {
        self.update_ui();

        self.new_btn.connect_clicked(move |_| on_new());
        self.open_btn.connect_clicked(move |_| on_open());

        self.window.show_all();
        gtk::main();
    }

    pub fn pubkey_selection(
        &self,
    ) -> Option<(String, gtk::TreeModel, gtk::TreeIter)> {
        self.pubkey_tree.get_selection().get_selected().and_then(
            |(model, iter)| {
                model
                    .get_value(&iter, 1)
                    .get::<String>()
                    .ok()
                    .flatten()
                    .map(|keyname| (keyname, model, iter))
            },
        )
    }

    pub fn descriptor_selection(
        &self,
    ) -> Option<(String, gtk::TreeModel, gtk::TreeIter)> {
        self.descriptor_tree
            .get_selection()
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

    pub fn utxo_selection(
        utxo_tree: &gtk::TreeView,
    ) -> Option<(OutPoint, gtk::TreeModel, gtk::TreeIter)> {
        utxo_tree
            .get_selection()
            .get_selected()
            .map(|(model, iter)| {
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
                vout.map(|vout| {
                    txid.map(|txid| (OutPoint { txid, vout }, model, iter))
                })
                .flatten()
            })
            .flatten()
    }

    pub fn update_ui(&self) {}
}
