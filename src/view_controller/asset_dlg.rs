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
use std::str::FromStr;

use lnpbp::bitcoin::OutPoint;

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

    issue_amount: RefCell<u64>,
    inflation_cap_saved: RefCell<f64>,
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
        let equal_radio = builder.get_object("equalRadio")?;
        let custom_radio = builder.get_object("customRadio")?;
        let custom_spin = builder.get_object("customSpin")?;
        let custom_adj = builder.get_object("customAdj")?;

        let issue_cap_display = builder.get_object("issueAcc")?;
        let inflation_cap_display = builder.get_object("inflationAcc")?;
        let total_cap_display = builder.get_object("totalAcc")?;
        let issue_amount_display = builder.get_object("issueAtomic")?;
        let inflation_amount_display = builder.get_object("inflationAtomic")?;
        let total_amount_display = builder.get_object("totalAtomic")?;
        let ticker1_label = builder.get_object("ticker1Label")?;
        let ticker2_label = builder.get_object("ticker2Label")?;
        let ticker3_label = builder.get_object("ticker3Label")?;
        let ticker4_label = builder.get_object("ticker4Label")?;

        let me = Rc::new(Self {
            dialog: glade_load!(builder, "assetDlg")?,

            issue_amount: RefCell::new(0u64),
            inflation_cap_saved: RefCell::new(100000000_f64),
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

        for ctl in &[&me.fract_spin, &me.inflation_spin, &me.custom_spin] {
            ctl.connect_changed(clone!(@weak me => move |_| {
                me.update_ui();
            }));
        }

        for ctl in &[&me.equal_radio, &me.custom_radio] {
            ctl.connect_toggled(clone!(@weak me => move |_| {
                me.update_ui();
            }));
        }

        for ctl in &[&me.epoch_check, &me.inflation_check, &me.contract_check] {
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
                        u64::MAX as f64 / me.precision_divisor();
                }
                me.update_ui();
            }));

        for ctl in &[
            &me.allocation_tree.get_selection(),
            &me.inflation_tree.get_selection(),
        ] {
            ctl.connect_changed(clone!(@weak me => move |_| {
                me.update_ui();
            }));
        }

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
                            &format!("<equal part>"),
                        ]);
                        me.inflation.borrow_mut().insert(utxo, None);
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

    pub fn asset_ticker(&self) -> Option<String> {
        let ticker = self.ticker_field.get_text().to_string();
        if ticker.is_empty() {
            None
        } else {
            Some(ticker.to_uppercase())
        }
    }

    pub fn asset_title(&self) -> Option<String> {
        let title = self.title_field.get_text().to_string();
        if title.is_empty() {
            None
        } else {
            Some(title)
        }
    }

    pub fn asset_fractionals(&self) -> u8 {
        self.fract_spin.get_value_as_int() as u8
    }

    pub fn is_capped(&self) -> bool {
        if !self.inflation_check.get_active() {
            false
        } else {
            self.inflation_combo
                .get_active_id()
                .map(|id| &*id == "limited")
                .unwrap_or(false)
        }
    }

    pub fn precision_divisor(&self) -> f64 {
        10_u64.pow(self.asset_fractionals() as u32) as f64
    }

    pub fn issue_cap(&self) -> f64 {
        *self.issue_amount.borrow() as f64 / self.precision_divisor()
    }

    pub fn inflation_cap(&self) -> f64 {
        if !self.inflation_check.get_active() {
            0.0
        } else if self.is_capped() {
            self.inflation_spin.get_value()
        } else {
            u64::MAX as f64 / self.precision_divisor()
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
        self.issue_cap() + self.inflation_cap()
    }

    pub fn issue_amount(&self) -> u64 {
        *self.issue_amount.borrow()
    }

    pub fn inflation_amount(&self) -> u64 {
        (self.inflation_cap() * self.precision_divisor()) as u64
    }

    pub fn total_amount(&self) -> u64 {
        self.issue_amount() + self.inflation_amount()
    }

    pub fn selected_allocation(&self) -> Option<(UtxoEntry, f64)> {
        self.allocation_tree
            .get_selection()
            .get_selected()
            .and_then(|(model, iter)| {
                model.get_value(&iter, 3).get::<String>().ok().flatten()
            })
            .and_then(|s| OutPoint::from_str(&s).ok())
            .and_then(|outpoint| {
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
        self.inflation_tree
            .get_selection()
            .get_selected()
            .and_then(|(model, iter)| {
                model.get_value(&iter, 3).get::<String>().ok().flatten()
            })
            .and_then(|s| OutPoint::from_str(&s).ok())
            .and_then(|outpoint| {
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
        let ticker = self
            .asset_ticker()
            .map(|ticker| {
                self.ticker_field.set_text(&ticker);
                ticker
            })
            .unwrap_or(s!("???"));
        self.ticker1_label.set_text(&ticker);
        self.ticker2_label.set_text(&ticker);
        self.ticker3_label.set_text(&ticker);
        self.ticker4_label.set_text(&ticker);

        self.epoch_btn.set_sensitive(self.epoch_check.get_active());

        self.fract_adj
            .set_upper((self.inflation_amount() as f64).log10().floor());

        self.inflation_combo
            .set_sensitive(self.inflation_check.get_active());
        self.inflation_adj
            .set_upper(u64::MAX as f64 / self.precision_divisor());
        self.inflation_spin.set_sensitive(self.is_capped());
        if !self.is_capped() {
            self.inflation_spin
                .set_value(u64::MAX as f64 / self.precision_divisor());
        }

        self.contract_text
            .set_sensitive(self.contract_check.get_active());

        let allocation = self.selected_allocation();
        let inflation = self.selected_inflation();
        if !self.inflation_check.get_active() {
            self.inflation_tree.get_selection().unselect_all()
        }
        self.amount_edit_btn.set_sensitive(allocation.is_some());
        self.allocation_remove_btn
            .set_sensitive(allocation.is_some());
        self.inflation_tree
            .set_sensitive(self.inflation_check.get_active());
        self.inflation_add_btn
            .set_sensitive(self.inflation_check.get_active());
        self.inflation_remove_btn.set_sensitive(inflation.is_some());
        self.equal_radio.set_sensitive(inflation.is_some());
        self.custom_radio.set_sensitive(inflation.is_some());
        if let Some((_, cap)) = inflation {
            self.custom_spin
                .set_sensitive(inflation.is_some() && cap.is_some());
            match cap {
                Some(cap) => {
                    self.custom_spin.set_value(cap);
                    self.custom_adj.set_upper(
                        self.inflation_cap() - self.assigned_cap() + cap,
                    );
                }
                None => {
                    let cap = self.equal_inflation_cap();
                    self.custom_adj.set_upper(cap);
                    self.custom_spin.set_value(cap);
                }
            }
        }
        self.custom_spin.set_sensitive(
            inflation.is_some() && self.custom_radio.get_active(),
        );

        self.issue_cap_display
            .set_text(&self.issue_cap().to_string());
        self.inflation_cap_display
            .set_text(&self.inflation_cap().to_string());
        self.total_cap_display
            .set_text(&self.total_cap().to_string());
        self.issue_amount_display
            .set_text(&self.issue_amount().to_string());
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
