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
use std::collections::HashMap;
use std::rc::Rc;

use crate::model::{AssetGenesis, DescriptorGenerator, Document, UtxoEntry};
use crate::view_controller::UtxoSelectDlg;

static UI: &'static str = include_str!("../view/asset.glade");

#[derive(Debug, Display, From, Error)]
#[display(doc_comments)]
/// Errors from processing asset genesis data
pub enum Error {
    /// Temporary error
    None,
}

pub struct AssetDlg {
    dialog: gtk::Dialog,

    epoch_utxo: Rc<RefCell<Option<UtxoEntry>>>,
    allocation: Rc<RefCell<HashMap<UtxoEntry, f64>>>,
    inflation: Rc<RefCell<HashMap<UtxoEntry, f64>>>,

    msg_box: gtk::Box,
    msg_label: gtk::Label,
    msg_image: gtk::Image,

    id_field: gtk::Entry,
    chain_combo: gtk::ComboBox,
    ticker_field: gtk::Entry,
    title_field: gtk::Entry,
    fract_spin: gtk::SpinButton,
    fract_adj: gtk::Adjustment,
    epoch_check: gtk::CheckButton,
    epoch_btn: gtk::Button,
    epoch_field: gtk::Entry,
    inflation_check: gtk::CheckButton,
    inflation_combo: gtk::ComboBox,
    inflation_spin: gtk::SpinButton,
    inflation_adj: gtk::Adjustment,
    contract_check: gtk::CheckButton,
    contract_text: gtk::TextView,
    contract_buffer: gtk::TextBuffer,

    allocation_tree: gtk::TreeView,
    allocation_store: gtk::ListStore,
    inflation_tree: gtk::TreeView,
    inflation_store: gtk::ListStore,

    allocation_add_btn: gtk::ToolButton,
    allocation_remove_btn: gtk::ToolButton,
    amount_edit_btn: gtk::ToolButton,

    inflation_add_btn: gtk::ToolButton,
    inflation_remove_btn: gtk::ToolButton,
    cap_edit_btn: gtk::ToolButton,

    create_btn: gtk::Button,
    cancel_btn: gtk::Button,
}

impl AssetDlg {
    pub fn load_glade() -> Result<Rc<Self>, glade::Error> {
        let builder = gtk::Builder::from_string(UI);

        let create_btn = builder.get_object("create")?;
        let cancel_btn = builder.get_object("cancel")?;

        let msg_box = builder.get_object("messageBox")?;
        let msg_image = builder.get_object("messageImage")?;
        let msg_label = builder.get_object("messageLabel")?;

        let id_field = builder.get_object("idField")?;
        let chain_combo = builder.get_object("chainCombo")?;
        let ticker_field = builder.get_object("tickerField")?;
        let title_field = builder.get_object("titleField")?;
        let fract_spin = builder.get_object("fractSpin")?;
        let fract_adj = builder.get_object("fractAdj")?;
        let epoch_check = builder.get_object("epochCheck")?;
        let epoch_btn = builder.get_object("epochBtn")?;
        let epoch_field = builder.get_object("epochEntry")?;
        let inflation_check = builder.get_object("inflationCheck")?;
        let inflation_combo = builder.get_object("inflationCombo")?;
        let inflation_spin = builder.get_object("inflationSpin")?;
        let inflation_adj = builder.get_object("inflationAdj")?;
        let contract_check = builder.get_object("contractCheck")?;
        let contract_text = builder.get_object("contractText")?;
        let contract_buffer = builder.get_object("contractBuffer")?;

        let allocation_tree = builder.get_object("allocationTree")?;
        let allocation_store = builder.get_object("allocationStore")?;
        let inflation_tree = builder.get_object("inflationTree")?;
        let inflation_store = builder.get_object("inflationStore")?;

        let allocation_add_btn = builder.get_object("allocationAdd")?;
        let allocation_remove_btn = builder.get_object("allocationRemove")?;
        let amount_edit_btn = builder.get_object("amountEdit")?;
        let inflation_add_btn = builder.get_object("inflationAdd")?;
        let inflation_remove_btn = builder.get_object("inflationRemove")?;
        let cap_edit_btn = builder.get_object("capEdit")?;

        let me = Rc::new(Self {
            dialog: glade_load!(builder, "assetDlg")?,

            epoch_utxo: none!(),
            allocation: none!(),
            inflation: none!(),

            msg_box,
            msg_image,
            msg_label,

            id_field,
            chain_combo,
            ticker_field,
            title_field,
            fract_spin,
            fract_adj,
            epoch_check,
            epoch_btn,
            epoch_field,
            inflation_check,
            inflation_combo,
            inflation_spin,
            inflation_adj,
            contract_check,
            contract_text,
            contract_buffer,

            allocation_tree,
            allocation_store,
            inflation_tree,
            inflation_store,

            allocation_add_btn,
            allocation_remove_btn,
            amount_edit_btn,
            inflation_add_btn,
            inflation_remove_btn,
            cap_edit_btn,

            create_btn,
            cancel_btn,
        });

        Ok(me)
    }
}

impl AssetDlg {
    pub fn run(
        self: Rc<Self>,
        doc: Rc<RefCell<Document>>,
        asset_genesis: Option<AssetGenesis>,
        on_issue: impl Fn(AssetGenesis) + 'static,
        on_cancel: impl Fn() + 'static,
    ) {
        let me = self.clone();

        me.chain_combo
            .set_active_id(Some(&doc.borrow().chain().to_string()));

        me.update_ui();

        me.epoch_btn.connect_clicked(
            clone!(@weak me, @strong doc => move |_| {
                let utxo_dlg = UtxoSelectDlg::load_glade().expect("Must load");
                utxo_dlg.run(
                    doc.clone(),
                    clone!(@weak me, @strong doc => move |utxo| {
                        me.display_epoch_seal(
                            &utxo,
                            doc.borrow().descriptor_by_content(&utxo.descriptor_content)
                        );
                        *me.epoch_utxo.borrow_mut() = Some(utxo);
                    }),
                    || {},
                );

                me.update_ui()
            }),
        );

        me.allocation_add_btn.connect_clicked(
            clone!(@weak me, @strong doc => move |_| {
                let utxo_dlg = UtxoSelectDlg::load_glade().expect("Must load");
                utxo_dlg.run(
                    doc.clone(),
                    clone!(@weak me, @strong doc => move |utxo| {
                        let dg = doc
                            .borrow()
                            .descriptor_by_content(&utxo.descriptor_content);
                        me.allocation_store.insert_with_values(None, &[0, 1, 2, 3, 4], &[
                            &dg.as_ref().map(|g| g.descriptor()).unwrap_or(s!("-")),
                            &dg.as_ref().map(|g| g.name()).unwrap_or(s!("<unknown descriptor>")),
                            &utxo.amount,
                            &utxo.outpoint.to_string(),
                            &0,
                        ]);
                        me.allocation.borrow_mut().insert(utxo, 0.0);
                    }),
                    || {},
                );
                me.update_ui()
            }),
        );

        me.inflation_add_btn.connect_clicked(
            clone!(@weak me, @strong doc => move |_| {
                let utxo_dlg = UtxoSelectDlg::load_glade().expect("Must load");
                utxo_dlg.run(
                    doc.clone(),
                    clone!(@weak me, @strong doc => move |utxo| {
                        let dg = doc
                            .borrow()
                            .descriptor_by_content(&utxo.descriptor_content);
                        me.inflation_store.insert_with_values(None, &[0, 1, 2, 3, 4], &[
                            &dg.as_ref().map(|g| g.descriptor()).unwrap_or(s!("-")),
                            &dg.as_ref().map(|g| g.name()).unwrap_or(s!("<unknown descriptor>")),
                            &utxo.amount,
                            &utxo.outpoint.to_string(),
                            &0,
                        ]);
                        me.inflation.borrow_mut().insert(utxo, 0.0);
                    }),
                    || {},
                );
                me.update_ui()
            }),
        );

        me.cancel_btn
            .connect_clicked(clone!(@weak self as me => move |_| {
                me.dialog.close();
                on_cancel()
            }));

        me.create_btn.connect_clicked(
            clone!(@weak self as me => move |_| match self.asset_genesis() {
                Ok(asset_genesis) => {
                    me.dialog.close();
                    on_issue(asset_genesis);
                }
                Err(err) => {
                    me.display_error(err);
                    me.create_btn.set_sensitive(false);
                }
            }),
        );

        me.dialog.run();
        me.dialog.close();
    }

    pub fn asset_genesis(&self) -> Result<AssetGenesis, Error> {
        Err(Error::None)
    }

    pub fn display_info(&self, msg: impl ToString) {
        self.msg_label.set_text(&msg.to_string());
        self.msg_image.set_from_icon_name(
            Some("dialog-information"),
            gtk::IconSize::SmallToolbar,
        );
        self.msg_box.set_visible(true);
    }

    pub fn display_error(&self, msg: impl std::error::Error) {
        self.msg_label.set_text(&msg.to_string());
        self.msg_image.set_from_icon_name(
            Some("dialog-error"),
            gtk::IconSize::SmallToolbar,
        );
        self.msg_box.set_visible(true);
    }

    pub fn display_epoch_seal(
        &self,
        utxo: &UtxoEntry,
        descriptor_generator: Option<DescriptorGenerator>,
    ) {
        let name = match descriptor_generator {
            Some(descriptor_generator) => {
                format!(
                    "{}: {} ({} sats)",
                    descriptor_generator.name(),
                    utxo.outpoint,
                    utxo.amount
                )
            }
            None => format!(
                "<unknown descriptor>: {} ({} sats)",
                utxo.outpoint, utxo.amount
            ),
        };
        self.epoch_field.set_text(&name);
    }

    pub fn update_ui(&self) {
        match self.update_ui_internal() {
            Ok(None) => {
                self.msg_box.set_visible(false);
                self.create_btn.set_sensitive(true);
            }
            Ok(Some(msg)) => {
                self.display_info(msg);
                self.create_btn.set_sensitive(true);
            }
            Err(err) => {
                self.display_error(err);
                self.create_btn.set_sensitive(false);
            }
        }
    }

    pub fn update_ui_internal(&self) -> Result<Option<String>, Error> {
        Ok(None)
    }
}
