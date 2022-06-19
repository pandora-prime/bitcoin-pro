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
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::str::FromStr;

use bitcoin::OutPoint;
use lnpbp::Chain;
use rgb::{AtomicValue, ContractId, Genesis, ToBech32};

use crate::model::{DescriptorAccount, Document, UtxoEntry};
use crate::view_controller::UtxoSelectDlg;

static UI: &str = include_str!("../view/asset.glade");

#[derive(Debug, Display, From, Error)]
#[display(doc_comments)]
/// Errors from processing asset genesis data
pub enum Error {
    /// Error from RGB20 procedures
    #[from]
    #[display(inner)]
    Rgb20(rgb20::Error),
}

pub struct AssetDlg {
    dialog: gtk::Dialog,

    chain: RefCell<Chain>,
    inflation_cap_saved: RefCell<f64>,
    renomination_utxo: Rc<RefCell<Option<UtxoEntry>>>,
    epoch_utxo: Rc<RefCell<Option<UtxoEntry>>>,
    allocation: Rc<RefCell<HashMap<UtxoEntry, f64>>>,
    inflation: Rc<RefCell<HashMap<UtxoEntry, Option<f64>>>>,

    msg_box: gtk::Box,
    msg_label: gtk::Label,
    msg_image: gtk::Image,

    id_field: gtk::Entry,
    chain_combo: gtk::ComboBox,
    ticker_field: gtk::Entry,
    title_field: gtk::Entry,
    fract_spin: gtk::SpinButton,
    fract_adj: gtk::Adjustment,
    renomen_check: gtk::CheckButton,
    renomen_btn: gtk::Button,
    renomen_field: gtk::Entry,
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

    inflation_add_btn: gtk::ToolButton,
    inflation_remove_btn: gtk::ToolButton,
    amount_spin: gtk::SpinButton,
    amount_adj: gtk::Adjustment,
    equal_radio: gtk::RadioToolButton,
    custom_radio: gtk::RadioToolButton,
    custom_spin: gtk::SpinButton,
    custom_adj: gtk::Adjustment,

    issue_cap_display: gtk::Entry,
    inflation_cap_display: gtk::Entry,
    total_cap_display: gtk::Entry,
    issue_amount_display: gtk::Entry,
    inflation_amount_display: gtk::Entry,
    total_amount_display: gtk::Entry,
    ticker1_label: gtk::Label,
    ticker2_label: gtk::Label,
    ticker3_label: gtk::Label,
    ticker4_label: gtk::Label,

    create_btn: gtk::Button,
    cancel_btn: gtk::Button,
}

impl AssetDlg {
    pub fn load_glade() -> Option<Rc<Self>> {
        let builder = gtk::Builder::from_string(UI);

        let create_btn = builder.object("create")?;
        let cancel_btn = builder.object("cancel")?;

        let msg_box = builder.object("messageBox")?;
        let msg_image = builder.object("messageImage")?;
        let msg_label = builder.object("messageLabel")?;

        let id_field = builder.object("idField")?;
        let chain_combo = builder.object("chainCombo")?;
        let ticker_field = builder.object("tickerField")?;
        let title_field = builder.object("titleField")?;
        let fract_spin = builder.object("fractSpin")?;
        let fract_adj = builder.object("fractAdj")?;
        let renomen_check = builder.object("renomenCheck")?;
        let renomen_btn = builder.object("renomenBtn")?;
        let renomen_field = builder.object("renomenEntry")?;
        let epoch_check = builder.object("epochCheck")?;
        let epoch_btn = builder.object("epochBtn")?;
        let epoch_field = builder.object("epochEntry")?;
        let inflation_check = builder.object("inflationCheck")?;
        let inflation_combo = builder.object("inflationCombo")?;
        let inflation_spin = builder.object("inflationSpin")?;
        let inflation_adj = builder.object("inflationAdj")?;
        let contract_check = builder.object("contractCheck")?;
        let contract_text = builder.object("contractText")?;
        let contract_buffer = builder.object("contractBuffer")?;

        let allocation_tree = builder.object("allocationTree")?;
        let allocation_store = builder.object("allocationStore")?;
        let inflation_tree = builder.object("inflationTree")?;
        let inflation_store = builder.object("inflationStore")?;

        let allocation_add_btn = builder.object("allocationAdd")?;
        let allocation_remove_btn = builder.object("allocationRemove")?;
        let inflation_add_btn = builder.object("inflationAdd")?;
        let inflation_remove_btn = builder.object("inflationRemove")?;
        let amount_spin = builder.object("amountSpin")?;
        let amount_adj = builder.object("amountAdj")?;
        let equal_radio = builder.object("equalRadio")?;
        let custom_radio = builder.object("customRadio")?;
        let custom_spin = builder.object("customSpin")?;
        let custom_adj = builder.object("customAdj")?;

        let issue_cap_display = builder.object("issueAcc")?;
        let inflation_cap_display = builder.object("inflationAcc")?;
        let total_cap_display = builder.object("totalAcc")?;
        let issue_amount_display = builder.object("issueAtomic")?;
        let inflation_amount_display = builder.object("inflationAtomic")?;
        let total_amount_display = builder.object("totalAtomic")?;
        let ticker1_label = builder.object("ticker1Label")?;
        let ticker2_label = builder.object("ticker2Label")?;
        let ticker3_label = builder.object("ticker3Label")?;
        let ticker4_label = builder.object("ticker4Label")?;

        let me = Rc::new(Self {
            dialog: glade_load!(builder, "assetDlg").ok()?,

            chain: RefCell::new(Chain::default()),
            inflation_cap_saved: RefCell::new(100000000_f64),
            renomination_utxo: none!(),
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
            renomen_check,
            renomen_btn,
            renomen_field,
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
            inflation_add_btn,
            inflation_remove_btn,
            amount_spin,
            amount_adj,
            equal_radio,
            custom_radio,
            custom_spin,
            custom_adj,

            issue_cap_display,
            inflation_cap_display,
            total_cap_display,
            issue_amount_display,
            inflation_amount_display,
            total_amount_display,
            ticker1_label,
            ticker2_label,
            ticker3_label,
            ticker4_label,

            create_btn,
            cancel_btn,
        });

        for ctl in &[&me.ticker_field, &me.title_field] {
            ctl.connect_changed(clone!(@weak me => move |_| {
                me.update_ui();
            }));
        }

        for ctl in &[&me.fract_spin, &me.inflation_spin] {
            ctl.connect_changed(clone!(@weak me => move |_| {
                me.update_ui();
            }));
        }

        for ctl in &[
            &me.renomen_check,
            &me.epoch_check,
            &me.inflation_check,
            &me.contract_check,
        ] {
            ctl.connect_toggled(clone!(@weak me => move |_| {
                me.update_ui();
            }));
        }

        me.inflation_combo
            .connect_changed(clone!(@weak me => move |_| {
                if me.is_capped() {
                    me.inflation_spin.set_value(*me.inflation_cap_saved.borrow());
                } else {
                    *me.inflation_cap_saved.borrow_mut() =
                        me.max_cap();
                }
                me.update_ui();
            }));

        for ctl in &[
            &me.allocation_tree.selection(),
            &me.inflation_tree.selection(),
        ] {
            ctl.connect_changed(clone!(@weak me => move |_| {
                if let Some((_, amount)) = me.selected_allocation() {
                    me.amount_spin.set_value(amount);
                } else {
                    me.amount_spin.set_value(0.0);
                }
                if let Some((_, cap)) = me.selected_inflation() {
                    me.equal_radio.set_active(cap.is_none());
                    me.custom_radio.set_active(cap.is_some());
                    match cap {
                        Some(cap) => me.custom_spin.set_value(cap),
                        None => me.custom_spin.set_value(me.equal_inflation_cap()),
                    }
                } else {
                    me.custom_spin.set_value(0.0)
                }
                me.update_ui();
            }));
        }

        me.amount_spin.connect_value_changed(clone!(@weak me => move |_| {
            if let Some((outpoint, _, iter)) = me.selected_allocation_model() {
                let value = me.amount_spin.value();
                if let Some((_, amount)) = me.allocation
                    .borrow_mut()
                    .iter_mut()
                    .find(|(utxo, _)| utxo.outpoint == outpoint)
                    { *amount = value }
                me.allocation_store.set_value(&iter, 4, &value.to_value());
            }
            me.update_ui();
        }));

        me.custom_spin.connect_value_changed(clone!(@weak me => move |_| {
            if let Some((outpoint, _, iter)) = me.selected_inflation_model() {
                let value = me.custom_spin.value();
                    let value = me.inflation
                    .borrow_mut()
                    .iter_mut()
                    .find(|(utxo, _)| utxo.outpoint == outpoint)
                    .and_then(|(_, amount)| amount.as_mut())
                    .map(|amount| {
                        *amount = value;
                        *amount
                    });
                match value {
                    Some(value) => {
                        me.inflation_store.set_value(&iter, 4, &value.to_value());
                    }
                    None => {
                        me.inflation_store.set_value(&iter, 4, &"<equal part>".to_value());
                        me.custom_spin.set_value(me.equal_inflation_cap());
                    }
                }
            }
            me.update_ui();
        }));

        me.equal_radio.connect_toggled(clone!(@weak me => move |_| {
            if let Some((outpoint, _, iter)) = me.selected_inflation_model() {
                if let Some((_, amount)) = me.inflation
                    .borrow_mut()
                    .iter_mut()
                    .find(|(utxo, _)| utxo.outpoint == outpoint)
                    { *amount = None }
                me.inflation_store.set_value(&iter, 4, &"<equal part>".to_value());
                me.custom_spin.set_value(me.equal_inflation_cap());
            }
        }));

        me.custom_radio
            .connect_toggled(clone!(@weak me => move |_| {
                if let Some((outpoint, _, _)) = me.selected_inflation_model() {
                    let value = me.inflation
                        .borrow_mut()
                        .iter_mut()
                        .find(|(utxo, _)| utxo.outpoint == outpoint)
                        .and_then(|(_, amount)| {
                            if amount.is_none() {
                                *amount = Some(0.0);
                            }
                            *amount
                        });
                    if let Some(value) = value { me.custom_spin.set_value(value) }
                }
            }));

        me.allocation_remove_btn
            .connect_clicked(clone!(@weak me => move |_| {
                if let Some((outpoint, _, iter)) = me.selected_allocation_model() {
                    let utxo = if let Some((utxo, _)) = me.allocation
                        .borrow()
                        .iter()
                        .find(|(utxo, _)| utxo.outpoint == outpoint)
                    {
                        utxo.clone()
                    } else {
                        return
                    };
                    me.allocation.borrow_mut().remove(&utxo);
                    me.allocation_store.remove(&iter);
                }
                me.update_ui();
            }));

        me.inflation_remove_btn
            .connect_clicked(clone!(@weak me => move |_| {
                if let Some((outpoint, _, iter)) = me.selected_inflation_model() {
                    let utxo = if let Some((utxo, _)) = me.inflation
                        .borrow()
                        .iter()
                        .find(|(utxo, _)| utxo.outpoint == outpoint)
                    {
                        utxo.clone()
                    } else {
                        return
                    };
                    me.inflation.borrow_mut().remove(&utxo);
                    me.inflation_store.remove(&iter);
                }
                me.update_ui();
            }));

        Some(me)
    }
}

impl AssetDlg {
    pub fn run(
        self: Rc<Self>,
        doc: Rc<RefCell<Document>>,
        contract_id: Option<ContractId>,
        on_issue: impl Fn(rgb20::Asset, Genesis) + 'static,
        on_cancel: impl Fn() + 'static,
    ) {
        let me = self.clone();

        if let Some(contract_id) = contract_id {
            self.apply_contract_id(doc.clone(), contract_id);
        }

        *me.chain.borrow_mut() = doc.borrow().chain().clone();
        me.chain_combo
            .set_active_id(Some(&me.chain.borrow().to_string()));

        me.update_ui();

        me.renomen_btn.connect_clicked(
            clone!(@weak me, @strong doc => move |_| {
                let utxo_dlg = UtxoSelectDlg::load_glade().expect("Must load");
                utxo_dlg.run(
                    doc.clone(),
                    clone!(@weak me, @strong doc => move |utxo| {
                        me.display_renomination_seal(
                            &utxo,
                            doc.borrow().descriptor_by_template(&utxo.descriptor_template)
                        );
                        *me.renomination_utxo.borrow_mut() = Some(utxo);
                    }),
                    || {},
                );

                me.update_ui()
            }),
        );

        me.epoch_btn.connect_clicked(
            clone!(@weak me, @strong doc => move |_| {
                let utxo_dlg = UtxoSelectDlg::load_glade().expect("Must load");
                utxo_dlg.run(
                    doc.clone(),
                    clone!(@weak me, @strong doc => move |utxo| {
                        me.display_epoch_seal(
                            &utxo,
                            doc.borrow().descriptor_by_template(&utxo.descriptor_template)
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
                            .descriptor_by_template(&utxo.descriptor_template);
                        me.allocation_store.insert_with_values(None, &[
                            (0, &dg.as_ref().map(|g| g.descriptor()).unwrap_or_else(|| s!("-"))),
                            (1, &dg.as_ref().map(|g| g.name()).unwrap_or_else(|| s!("<unknown descriptor>"))),
                            (2, &utxo.amount),
                            (3, &utxo.outpoint.to_string()),
                            (4, &0)
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
                            .descriptor_by_template(&utxo.descriptor_template);
                        me.inflation_store.insert_with_values(None, &[
                            (0, &dg.as_ref().map(|g| g.descriptor()).unwrap_or_else(|| s!("-"))),
                            (1, &dg.as_ref().map(|g| g.name()).unwrap_or_else(|| s!("<unknown descriptor>"))),
                            (2, &utxo.amount),
                            (3, &utxo.outpoint.to_string()),
                            (4, &"<equal part>")
                        ]);
                        me.inflation.borrow_mut().insert(utxo, None);
                        me.custom_spin.set_value(me.equal_inflation_cap());
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
                Ok((asset, genesis)) => {
                    me.dialog.close();
                    on_issue(asset, genesis);
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

    pub fn apply_contract_id(
        &self,
        doc: Rc<RefCell<Document>>,
        contract_id: ContractId,
    ) {
        let (_asset, _genesis) = if let Some((asset, genesis)) =
            doc.borrow().asset_by_id(contract_id)
        {
            (asset, genesis)
        } else {
            return;
        };

        self.id_field.set_text(&contract_id.to_bech32_string());
        // TODO: Implement
    }

    pub fn asset_genesis(&self) -> Result<(rgb20::Asset, Genesis), Error> {
        Ok(rgb20::issue(
            self.chain.borrow().clone(),
            self.asset_ticker().unwrap_or_default(),
            self.asset_title().unwrap_or_default(),
            self.asset_contract(),
            self.asset_fractionals(),
            self.asset_allocation(),
            self.asset_inflation(),
            self.asset_renomination(),
            self.asset_epoch(),
        )?)
    }

    pub fn asset_ticker(&self) -> Option<String> {
        let ticker = self.ticker_field.text().to_string();
        if ticker.is_empty() {
            None
        } else {
            Some(ticker.to_uppercase())
        }
    }

    pub fn asset_title(&self) -> Option<String> {
        let title = self.title_field.text().to_string();
        if title.is_empty() {
            None
        } else {
            Some(title)
        }
    }

    pub fn asset_contract(&self) -> Option<String> {
        if self.contract_check.is_active() {
            self.contract_buffer
                .text(
                    &self.contract_buffer.start_iter(),
                    &self.contract_buffer.end_iter(),
                    false,
                )
                .and_then(|text| {
                    let text = text.to_string();
                    if text.is_empty() {
                        None
                    } else {
                        Some(text)
                    }
                })
        } else {
            None
        }
    }

    pub fn asset_fractionals(&self) -> u8 {
        self.fract_spin.value_as_int() as u8
    }

    pub fn asset_allocation(&self) -> Vec<(OutPoint, AtomicValue)> {
        self.allocation
            .borrow()
            .iter()
            .map(|(utxo, amount)| {
                (utxo.outpoint, (amount * self.precision_divisor()) as u64)
            })
            .collect()
    }

    pub fn asset_inflation(&self) -> BTreeMap<OutPoint, AtomicValue> {
        if self.inflation_check.is_active() {
            self.inflation
                .borrow()
                .iter()
                .map(|(utxo, maybe_amount)| {
                    (
                        utxo.outpoint,
                        (maybe_amount.unwrap_or(self.equal_inflation_cap())
                            * self.precision_divisor())
                            as u64,
                    )
                })
                .collect()
        } else {
            bmap! {}
        }
    }

    pub fn asset_renomination(&self) -> Option<OutPoint> {
        self.renomination_utxo
            .borrow()
            .as_ref()
            .map(|utxo| utxo.outpoint)
    }

    pub fn asset_epoch(&self) -> Option<OutPoint> {
        self.epoch_utxo.borrow().as_ref().map(|utxo| utxo.outpoint)
    }

    pub fn is_capped(&self) -> bool {
        if !self.inflation_check.is_active() {
            false
        } else {
            self.inflation_combo
                .active_id()
                .map(|id| &*id == "limited")
                .unwrap_or(false)
        }
    }

    pub fn max_cap(&self) -> f64 {
        (u64::MAX - 1) as f64 / self.precision_divisor()
    }

    pub fn precision_divisor(&self) -> f64 {
        10_u64.pow(self.asset_fractionals() as u32) as f64
    }

    pub fn inflation_cap(&self) -> f64 {
        if !self.inflation_check.is_active() {
            0.0
        } else if self.is_capped() {
            self.inflation_spin.value()
        } else {
            self.max_cap() - self.assigned_cap() - 1.0 // TODO: Fix this
        }
    }

    pub fn assigned_cap(&self) -> f64 {
        self.allocation
            .borrow()
            .iter()
            .fold(0.0f64, |sum, (_, amount)| sum + amount)
    }

    pub fn inflation_sum(&self) -> f64 {
        self.inflation
            .borrow()
            .iter()
            .fold(0.0f64, |sum, (_, amount)| sum + amount.unwrap_or(0.0f64))
    }

    pub fn equal_inflation_cap(&self) -> f64 {
        let len = self.inflation.borrow().len();
        let len = if len == 0 { 1 } else { len };
        (self.inflation_cap() - self.inflation_sum()) / len as f64
    }

    pub fn total_cap(&self) -> f64 {
        self.assigned_cap() + self.inflation_cap()
    }

    pub fn assigned_amount(&self) -> u64 {
        (self.assigned_cap() * self.precision_divisor()) as u64
    }

    pub fn inflation_amount(&self) -> u64 {
        (self.inflation_cap() * self.precision_divisor()) as u64
    }

    pub fn total_amount(&self) -> u64 {
        self.assigned_amount() + self.inflation_amount()
    }

    fn selected_allocation_model(
        &self,
    ) -> Option<(OutPoint, gtk::TreeModel, gtk::TreeIter)> {
        self.allocation_tree
            .selection()
            .selected()
            .and_then(|(model, iter)| {
                model
                    .value(&iter, 3)
                    .get::<String>()
                    .ok()
                    .and_then(|s| OutPoint::from_str(&s).ok())
                    .map(|outpoint| (outpoint, model, iter))
            })
    }

    fn selected_inflation_model(
        &self,
    ) -> Option<(OutPoint, gtk::TreeModel, gtk::TreeIter)> {
        self.inflation_tree
            .selection()
            .selected()
            .and_then(|(model, iter)| {
                model
                    .value(&iter, 3)
                    .get::<String>()
                    .ok()
                    .and_then(|s| OutPoint::from_str(&s).ok())
                    .map(|outpoint| (outpoint, model, iter))
            })
    }

    pub fn selected_allocation(&self) -> Option<(UtxoEntry, f64)> {
        self.selected_allocation_model().and_then(|(outpoint, ..)| {
            self.allocation.borrow().iter().find_map(|(utxo, amount)| {
                if utxo.outpoint == outpoint {
                    Some((utxo.clone(), *amount))
                } else {
                    None
                }
            })
        })
    }

    pub fn selected_inflation(&self) -> Option<(UtxoEntry, Option<f64>)> {
        self.selected_inflation_model().and_then(|(outpoint, ..)| {
            self.inflation.borrow().iter().find_map(|(utxo, amount)| {
                if utxo.outpoint == outpoint {
                    Some((utxo.clone(), *amount))
                } else {
                    None
                }
            })
        })
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

    pub fn display_renomination_seal(
        &self,
        utxo: &UtxoEntry,
        descriptor_generator: Option<DescriptorAccount>,
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
        self.renomen_field.set_text(&name);
    }

    pub fn display_epoch_seal(
        &self,
        utxo: &UtxoEntry,
        descriptor_generator: Option<DescriptorAccount>,
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
        let ticker = self
            .asset_ticker()
            .map(|ticker| {
                self.ticker_field.set_text(&ticker);
                ticker
            })
            .unwrap_or_else(|| s!("???"));
        self.ticker1_label.set_text(&ticker);
        self.ticker2_label.set_text(&ticker);
        self.ticker3_label.set_text(&ticker);
        self.ticker4_label.set_text(&ticker);

        self.renomen_btn
            .set_sensitive(self.renomen_check.is_active());
        self.epoch_btn.set_sensitive(self.epoch_check.is_active());

        self.inflation_combo
            .set_sensitive(self.inflation_check.is_active());
        self.inflation_adj.set_upper(self.max_cap());
        self.inflation_spin.set_sensitive(self.is_capped());
        if !self.is_capped() {
            self.inflation_spin.set_value(self.max_cap());
        }

        self.contract_text
            .set_sensitive(self.contract_check.is_active());

        let allocation = self.selected_allocation();
        let inflation = self.selected_inflation();
        if !self.inflation_check.is_active() {
            self.inflation_tree.selection().unselect_all()
        }
        self.allocation_remove_btn
            .set_sensitive(allocation.is_some());
        self.inflation_tree
            .set_sensitive(self.inflation_check.is_active());
        self.inflation_add_btn
            .set_sensitive(self.inflation_check.is_active());
        self.inflation_remove_btn.set_sensitive(inflation.is_some());
        self.amount_spin.set_sensitive(allocation.is_some());
        self.custom_spin.set_sensitive(
            inflation.is_some() && self.custom_radio.is_active(),
        );
        self.equal_radio.set_sensitive(inflation.is_some());
        self.custom_radio.set_sensitive(inflation.is_some());
        if let Some((_, amount)) = allocation {
            self.amount_adj
                .set_upper(self.max_cap() - self.assigned_cap() + amount);
        }
        if let Some((_, cap)) = inflation {
            match cap {
                Some(cap) => {
                    self.equal_radio.set_active(false);
                    self.custom_radio.set_active(true);
                    self.custom_adj.set_upper(
                        self.inflation_cap() - self.assigned_cap() + cap,
                    );
                }
                None => {
                    self.equal_radio.set_active(true);
                    self.custom_radio.set_active(false);
                    self.custom_adj
                        .set_upper(self.inflation_cap() - self.assigned_cap());
                }
            }
        }

        self.issue_cap_display
            .set_text(&self.assigned_cap().to_string());
        self.inflation_cap_display
            .set_text(&self.inflation_cap().to_string());
        self.total_cap_display
            .set_text(&self.total_cap().to_string());
        self.issue_amount_display
            .set_text(&self.assigned_amount().to_string());
        self.inflation_amount_display
            .set_text(&self.inflation_amount().to_string());
        self.total_amount_display
            .set_text(&self.total_amount().to_string());

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
