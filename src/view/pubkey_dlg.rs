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
use std::ops::RangeInclusive;
use std::rc::Rc;
use std::str::FromStr;

use lnpbp::bitcoin::util::bip32::{
    self, ChildNumber, DerivationPath, ExtendedPrivKey, ExtendedPubKey,
};
use lnpbp::bitcoin::util::{base58, key};
use lnpbp::{bitcoin, secp256k1};

use crate::model::{
    DerivationComponents, DerivationRange, TrackingAccount, TrackingKey,
};

static UI: &'static str = include_str!("../../ui/pubkey.glade");

#[derive(Debug, Display, From, Error)]
#[display(doc_comments)]
/// Errors from processing public key derivation data
pub enum Error {
    /// Wrong public key data
    #[display("{0}")]
    #[from]
    Secp(secp256k1::Error),

    /// BIP32-specific error
    #[display("{0}")]
    #[from]
    Key(key::Error),

    /// BIP32-specific error
    #[display("{0}")]
    #[from]
    Bip32(bip32::Error),

    /// Wrong extended public key data
    #[display("{0}")]
    #[from]
    Base58(base58::Error),

    /// Unable to parse '{0}' as index at position {1}
    WrongIndexNumber(String, usize),

    /// Unable to parse '{0}' as range at position {1}
    WrongRange(String, usize),

    /// Empty range specifier position {0}
    EmptyRange(usize),

    /// Unsupported blockchain
    UnsupportedBlockchain,

    /// You need to specify blockchain type
    UnspecifiedBlockchain,

    /// You must provide a non-empty name
    EmptyName,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Display)]
#[display(Debug)]
pub enum PkType {
    Single,
    Hd,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Display)]
#[display(Debug)]
pub enum DeriveType {
    Bip44,
    Custom,
}

//#[display(Glade)]
//#[glade(file = "../ui/asset_issue.glade")]
pub struct PubkeyDlg {
    //#[glade(id = "assetIssue")]
    dialog: gtk::Dialog,
    msg_box: gtk::Box,
    msg_label: gtk::Label,
    msg_image: gtk::Image,
    save_btn: gtk::Button,
    cancel_btn: gtk::Button,

    name_field: gtk::Entry,
    pubkey_field: gtk::Entry,
    xpub_field: gtk::Entry,

    sk_radio: gtk::RadioButton,
    hd_radio: gtk::RadioButton,

    bip44_radio: gtk::RadioButton,
    custom_radio: gtk::RadioButton,

    purpose_combo: gtk::ComboBox,
    purpose_index: gtk::SpinButton,
    purpose_chk: gtk::CheckButton,

    asset_combo: gtk::ComboBox,
    asset_index: gtk::SpinButton,
    asset_chk: gtk::CheckButton,

    account_index: gtk::SpinButton,
    account_chk: gtk::CheckButton,

    change_combo: gtk::ComboBox,
    change_index: gtk::SpinButton,
    change_chk: gtk::CheckButton,

    range_chk: gtk::CheckButton,
    range_field: gtk::Entry,
    derivation_field: gtk::Entry,

    network_combo: gtk::ComboBox,
    offset_index: gtk::SpinButton,
    offset_chk: gtk::CheckButton,

    xpubid_display: gtk::Entry,
    fingerprint_display: gtk::Entry,
    derivation_display: gtk::Entry,
    descriptor_display: gtk::Entry,
    xpub_display: gtk::Entry,

    uncompressed_display: gtk::Entry,
    compressed_display: gtk::Entry,
    xcoordonly_display: gtk::Entry,

    pkh_display: gtk::Entry,
    wpkh_display: gtk::Entry,
    wpkh_sh_display: gtk::Entry,
    taproot_display: gtk::Entry,

    bech_display: gtk::Entry,
}

impl glade::View for PubkeyDlg {
    fn load_glade() -> Result<Rc<RefCell<Self>>, glade::Error> {
        let builder = gtk::Builder::from_string(UI);

        let save_btn = builder
            .get_object("save")
            .ok_or(glade::Error::WidgetNotFound)?;
        let cancel_btn = builder
            .get_object("cancel")
            .ok_or(glade::Error::WidgetNotFound)?;

        let msg_box = builder
            .get_object("messageBox")
            .ok_or(glade::Error::WidgetNotFound)?;
        let msg_image = builder
            .get_object("messageImage")
            .ok_or(glade::Error::WidgetNotFound)?;
        let msg_label = builder
            .get_object("messageLabel")
            .ok_or(glade::Error::WidgetNotFound)?;

        let name_field = builder
            .get_object("nameField")
            .ok_or(glade::Error::WidgetNotFound)?;
        let pubkey_field = builder
            .get_object("pubkeyField")
            .ok_or(glade::Error::WidgetNotFound)?;
        let xpub_field = builder
            .get_object("xpubField")
            .ok_or(glade::Error::WidgetNotFound)?;
        let sk_radio = builder
            .get_object("singleKey")
            .ok_or(glade::Error::WidgetNotFound)?;
        let hd_radio = builder
            .get_object("hdKey")
            .ok_or(glade::Error::WidgetNotFound)?;
        let bip44_radio = builder
            .get_object("deriveBip44")
            .ok_or(glade::Error::WidgetNotFound)?;
        let custom_radio = builder
            .get_object("deriveCustom")
            .ok_or(glade::Error::WidgetNotFound)?;

        let purpose_combo = builder
            .get_object("purposeCombo")
            .ok_or(glade::Error::WidgetNotFound)?;
        let purpose_index = builder
            .get_object("purposeCounter")
            .ok_or(glade::Error::WidgetNotFound)?;
        let purpose_chk = builder
            .get_object("purposeCheck")
            .ok_or(glade::Error::WidgetNotFound)?;

        let asset_combo = builder
            .get_object("assetCombo")
            .ok_or(glade::Error::WidgetNotFound)?;
        let asset_index = builder
            .get_object("assetCounter")
            .ok_or(glade::Error::WidgetNotFound)?;
        let asset_chk = builder
            .get_object("assetCheck")
            .ok_or(glade::Error::WidgetNotFound)?;

        let account_index = builder
            .get_object("accountCounter")
            .ok_or(glade::Error::WidgetNotFound)?;
        let account_chk = builder
            .get_object("accountCheck")
            .ok_or(glade::Error::WidgetNotFound)?;

        let change_combo = builder
            .get_object("changeCombo")
            .ok_or(glade::Error::WidgetNotFound)?;
        let change_index = builder
            .get_object("changeCounter")
            .ok_or(glade::Error::WidgetNotFound)?;
        let change_chk = builder
            .get_object("changeCheck")
            .ok_or(glade::Error::WidgetNotFound)?;

        let range_chk = builder
            .get_object("rangeCheck")
            .ok_or(glade::Error::WidgetNotFound)?;
        let range_field = builder
            .get_object("rangeField")
            .ok_or(glade::Error::WidgetNotFound)?;
        let derivation_field = builder
            .get_object("derivationField")
            .ok_or(glade::Error::WidgetNotFound)?;

        let network_combo = builder
            .get_object("blockchainCombo")
            .ok_or(glade::Error::WidgetNotFound)?;
        let offset_index = builder
            .get_object("exportIndex")
            .ok_or(glade::Error::WidgetNotFound)?;
        let offset_chk = builder
            .get_object("exportHCheck")
            .ok_or(glade::Error::WidgetNotFound)?;

        let xpubid_display = builder
            .get_object("xpubidDisplay")
            .ok_or(glade::Error::WidgetNotFound)?;
        let fingerprint_display = builder
            .get_object("fingerprintDisplay")
            .ok_or(glade::Error::WidgetNotFound)?;
        let derivation_display = builder
            .get_object("derivationDisplay")
            .ok_or(glade::Error::WidgetNotFound)?;
        let descriptor_display = builder
            .get_object("descriptorDisplay")
            .ok_or(glade::Error::WidgetNotFound)?;
        let xpub_display = builder
            .get_object("xpubDisplay")
            .ok_or(glade::Error::WidgetNotFound)?;

        let uncompressed_display = builder
            .get_object("uncompressedDisplay")
            .ok_or(glade::Error::WidgetNotFound)?;
        let compressed_display = builder
            .get_object("compressedDisplay")
            .ok_or(glade::Error::WidgetNotFound)?;
        let xcoordonly_display = builder
            .get_object("xonlyDisplay")
            .ok_or(glade::Error::WidgetNotFound)?;

        let pkh_display = builder
            .get_object("pkhDisplay")
            .ok_or(glade::Error::WidgetNotFound)?;
        let wpkh_display = builder
            .get_object("wpkhDisplay")
            .ok_or(glade::Error::WidgetNotFound)?;
        let wpkh_sh_display = builder
            .get_object("wpkhShDisplay")
            .ok_or(glade::Error::WidgetNotFound)?;
        let taproot_display = builder
            .get_object("taprootDisplay")
            .ok_or(glade::Error::WidgetNotFound)?;

        let bech_display = builder
            .get_object("bechDisplay")
            .ok_or(glade::Error::WidgetNotFound)?;

        let me = Rc::new(RefCell::new(Self {
            dialog: glade_load!(builder, "pubkeyDlg")?,
            save_btn,
            cancel_btn,
            msg_box,
            msg_image,
            msg_label,
            name_field,
            pubkey_field,
            xpub_field,
            sk_radio,
            hd_radio,
            bip44_radio,
            custom_radio,
            purpose_combo,
            purpose_index,
            purpose_chk,
            asset_combo,
            asset_index,
            asset_chk,
            account_index,
            account_chk,
            change_combo,
            change_index,
            change_chk,
            range_chk,
            range_field,
            derivation_field,
            network_combo,
            offset_index,
            offset_chk,
            xpubid_display,
            fingerprint_display,
            derivation_display,
            descriptor_display,
            xpub_display,
            uncompressed_display,
            compressed_display,
            xcoordonly_display,
            pkh_display,
            wpkh_display,
            wpkh_sh_display,
            taproot_display,
            bech_display,
        }));

        me.borrow()
            .name_field
            .connect_changed(clone!(@weak me => move |_| {
                let me = me.borrow();
                me.update_ui();
            }));

        me.borrow()
            .pubkey_field
            .connect_changed(clone!(@weak me => move |_| {
                let me = me.borrow();
                me.set_key_type(PkType::Single)
            }));

        for ctl in &[&me.borrow().xpub_field, &me.borrow().range_field] {
            ctl.connect_changed(clone!(@weak me => move |_| {
                let me = me.borrow();
                me.set_key_type(PkType::Hd)
            }));
        }

        me.borrow().derivation_field.connect_changed(
            clone!(@weak me => move |_| {
                let me = me.borrow();
                me.set_derive_type(DeriveType::Custom)
            }),
        );

        for ctl in &[
            &me.borrow().sk_radio,
            &me.borrow().hd_radio,
            &me.borrow().bip44_radio,
            &me.borrow().custom_radio,
        ] {
            ctl.connect_toggled(clone!(@weak me => move |_| {
                let me = me.borrow();
                me.update_ui()
            }));
        }

        for ctl in &[
            &me.borrow().purpose_combo,
            &me.borrow().asset_combo,
            &me.borrow().change_combo,
            &me.borrow().network_combo,
        ] {
            ctl.connect_changed(clone!(@weak me => move |_| {
                let me = me.borrow();
                me.update_ui()
            }));
        }

        for ctl in &[
            &me.borrow().purpose_index,
            &me.borrow().asset_index,
            &me.borrow().account_index,
            &me.borrow().change_index,
        ] {
            ctl.connect_changed(clone!(@weak me => move |_| {
                let me = me.borrow();
                me.set_derive_type(DeriveType::Bip44)
            }));
        }

        for ctl in &[
            &me.borrow().purpose_chk,
            &me.borrow().asset_chk,
            &me.borrow().account_chk,
            &me.borrow().change_chk,
            &me.borrow().range_chk,
        ] {
            ctl.connect_toggled(clone!(@weak me => move |_| {
                let me = me.borrow();
                me.set_derive_type(DeriveType::Bip44)
            }));
        }

        me.borrow()
            .offset_index
            .connect_changed(clone!(@weak me => move |_| {
                let me = me.borrow();
                me.update_ui();
            }));

        me.borrow()
            .offset_chk
            .connect_toggled(clone!(@weak me => move |_| {
                let me = me.borrow();
                me.update_ui();
            }));

        for ctl in &[
            &me.borrow().xpubid_display,
            &me.borrow().fingerprint_display,
            &me.borrow().derivation_display,
            &me.borrow().descriptor_display,
            &me.borrow().xpub_display,
            &me.borrow().uncompressed_display,
            &me.borrow().compressed_display,
            &me.borrow().xcoordonly_display,
            &me.borrow().pkh_display,
            &me.borrow().wpkh_display,
            &me.borrow().wpkh_sh_display,
            &me.borrow().taproot_display,
            &me.borrow().bech_display,
        ] {
            ctl.connect_icon_press(clone!(@weak ctl, @weak me => move |_, _, _| {
                let val = ctl.get_text();
                gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD)
                    .set_text(&val);
                me.borrow().display_info(format!("Value {} copied to clipboard", val));
            }));
        }

        Ok(me)
    }
}

impl PubkeyDlg {
    pub fn run(
        self: Rc<Self>,
        on_save: impl Fn(TrackingAccount) + 'static,
        on_cancel: impl Fn() + 'static,
    ) {
        let dlg = &self.dialog;
        let me = self;

        self.update_ui();

        self.cancel_btn
            .connect_clicked(clone!(@weak dlg => move |_| {
                dlg.hide();
                on_cancel()
            }));

        self.save_btn.connect_clicked(
            clone!(@weak me => move |_| match self.tracking_account() {
                Ok(tracking_account) => {
                    self.dialog.hide();
                    on_save(tracking_account);
                }
                Err(err) => {
                    self.display_error(err);
                    self.save_btn.set_sensitive(false);
                }
            }),
        );

        dlg.run();
        dlg.hide();
    }

    pub fn tracking_account(&self) -> Result<TrackingAccount, Error> {
        let key = if self.sk_radio.get_active() {
            TrackingKey::SingleKey(secp256k1::PublicKey::from_str(
                &self.pubkey_field.get_text(),
            )?)
        } else {
            TrackingKey::HdKeySet(self.derivation_components()?)
        };

        Ok(TrackingAccount {
            name: self.name_field.get_text().to_string(),
            key,
        })
    }

    pub fn derivation_path(&self) -> Result<DerivationPath, Error> {
        let derivation = if self.bip44_radio.get_active() {
            DerivationPath::from_str(&format!(
                "m/{}{}/{}{}/{}{}/{}{}",
                self.purpose_index.get_value() as u32,
                if self.purpose_chk.get_active() {
                    "'"
                } else {
                    ""
                },
                self.asset_index.get_value() as u32,
                if self.asset_chk.get_active() { "'" } else { "" },
                self.account_index.get_value() as u32,
                if self.account_chk.get_active() {
                    "'"
                } else {
                    ""
                },
                self.change_index.get_value() as u32,
                if self.change_chk.get_active() {
                    "'"
                } else {
                    ""
                }
            ))?
        } else {
            DerivationPath::from_str(&self.derivation_field.get_text())?
        };

        let s = format!(
            "m/{}{}",
            self.offset_index.get_value() as u32,
            if self.offset_chk.get_active() {
                "'"
            } else {
                ""
            }
        );
        Ok(derivation.extend(DerivationPath::from_str(&s)?))
    }

    pub fn derivation_components(&self) -> Result<DerivationComponents, Error> {
        let derivation = self.derivation_path()?;
        let mut path_iter = derivation.as_ref().into_iter().rev();
        let terminal_path: Vec<u32> = path_iter
            .by_ref()
            .map_while(|child| {
                if let ChildNumber::Normal { index } = child {
                    Some(index)
                } else {
                    None
                }
            })
            .cloned()
            .collect();
        let terminal_path = terminal_path.into_iter().rev().collect();
        let branch_path = path_iter.rev().cloned().collect();

        let index_ranges = self.index_ranges()?;

        if let Ok(master_priv) =
            ExtendedPrivKey::from_str(&self.xpub_field.get_text())
        {
            let master =
                ExtendedPubKey::from_private(&lnpbp::SECP256K1, &master_priv);
            let branch_xpub =
                master.derive_pub(&lnpbp::SECP256K1, &branch_path)?;
            Ok(DerivationComponents {
                branch_xpub,
                branch_source: (master.fingerprint(), branch_path),
                terminal_path,
                index_ranges,
            })
        } else if branch_path.as_ref().is_empty() {
            let branch_xpub =
                ExtendedPubKey::from_str(&self.xpub_field.get_text())?;
            Ok(DerivationComponents {
                branch_xpub,
                branch_source: (branch_xpub.fingerprint(), branch_path),
                terminal_path,
                index_ranges,
            })
        } else {
            Err(bip32::Error::CannotDeriveFromHardenedKey)?
        }
    }

    pub fn index_ranges(&self) -> Result<Vec<DerivationRange>, Error> {
        let mut index_ranges = vec![];
        for (pos, elem) in
            self.range_field.get_text().as_str().split(',').enumerate()
        {
            let mut split = elem.trim().split('-');
            let range = match (split.next(), split.next(), split.next()) {
                (None, None, None) => return Err(Error::EmptyRange(pos)),
                (Some(num), None, None) => {
                    let idx = num.parse().map_err(|_| {
                        Error::WrongIndexNumber(num.to_string(), pos)
                    })?;
                    RangeInclusive::new(idx, idx).into()
                }
                (Some(num1), Some(num2), None) => RangeInclusive::new(
                    num1.parse().map_err(|_| {
                        Error::WrongIndexNumber(num1.to_string(), pos)
                    })?,
                    num2.parse().map_err(|_| {
                        Error::WrongIndexNumber(num2.to_string(), pos)
                    })?,
                )
                .into(),
                _ => return Err(Error::WrongRange(elem.to_string(), pos)),
            };
            index_ranges.push(range);
        }
        Ok(index_ranges)
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

    pub fn set_key_type(&self, pk_type: PkType) {
        self.sk_radio.set_active(pk_type == PkType::Single);
        self.hd_radio.set_active(pk_type == PkType::Hd);
        self.update_ui();
    }

    pub fn set_derive_type(&self, derive_type: DeriveType) {
        self.bip44_radio
            .set_active(derive_type == DeriveType::Bip44);
        self.custom_radio
            .set_active(derive_type == DeriveType::Custom);
        self.update_ui();
    }

    pub fn update_ui(&self) {
        self.pubkey_field.set_sensitive(self.sk_radio.get_active());
        self.xpub_field.set_sensitive(self.hd_radio.get_active());
        self.derivation_field
            .set_sensitive(self.custom_radio.get_active());
        self.range_field.set_sensitive(self.range_chk.get_active());
        self.range_chk.set_sensitive(self.hd_radio.get_active());

        self.offset_index.set_sensitive(self.hd_radio.get_active());
        self.offset_chk.set_sensitive(self.hd_radio.get_active());

        for ctl in &[&self.bip44_radio, &self.custom_radio] {
            ctl.set_sensitive(self.hd_radio.get_active());
        }

        for ctl in &[&self.purpose_combo, &self.asset_combo, &self.change_combo]
        {
            ctl.set_sensitive(
                self.hd_radio.get_active() && self.bip44_radio.get_active(),
            );
        }

        for ctl in &[
            &self.purpose_index,
            &self.asset_index,
            &self.account_index,
            &self.change_index,
        ] {
            ctl.set_sensitive(
                self.hd_radio.get_active() && self.bip44_radio.get_active(),
            );
        }

        for ctl in &[
            &self.purpose_chk,
            &self.asset_chk,
            &self.account_chk,
            &self.change_chk,
        ] {
            ctl.set_sensitive(
                self.hd_radio.get_active() && self.bip44_radio.get_active(),
            );
        }

        if self.purpose_combo.get_active() != Some(4) {
            self.purpose_index.set_sensitive(false);
            self.purpose_chk.set_sensitive(false);
            self.purpose_index.set_value(
                self.purpose_combo
                    .get_active_id()
                    .map(|s| f64::from_str(&s).unwrap_or_default())
                    .unwrap_or_default(),
            );
            self.purpose_chk.set_active(true);
        }

        if self.asset_combo.get_active() != Some(4) {
            self.asset_index.set_sensitive(false);
            self.asset_chk.set_sensitive(false);
            self.asset_index.set_value(
                self.asset_combo
                    .get_active_id()
                    .map(|s| f64::from_str(&s).unwrap_or_default())
                    .unwrap_or_default(),
            );
            self.asset_chk.set_active(true);
        }

        if self.change_combo.get_active() != Some(2) {
            self.change_index.set_sensitive(false);
            self.change_chk.set_sensitive(false);
            self.change_index.set_value(
                self.change_combo
                    .get_active_id()
                    .map(|s| f64::from_str(&s).unwrap_or_default())
                    .unwrap_or_default(),
            );
            self.change_chk.set_active(false);
        }

        match self.update_ui_internal() {
            Ok(None) => {
                self.msg_box.set_visible(false);
                self.save_btn.set_sensitive(true);
            }
            Ok(Some(msg)) => {
                self.display_info(msg);
                self.save_btn.set_sensitive(true);
            }
            Err(err) => {
                self.display_error(err);
                self.save_btn.set_sensitive(false);
            }
        }
    }

    pub fn update_ui_internal(&self) -> Result<Option<String>, Error> {
        let network = match self.network_combo.get_active() {
            Some(0) => bitcoin::Network::Bitcoin,
            Some(1) => bitcoin::Network::Testnet,
            Some(2) => bitcoin::Network::Testnet,
            None => Err(Error::UnspecifiedBlockchain)?,
            _ => Err(Error::UnsupportedBlockchain)?,
        };

        let pk = if self.sk_radio.get_active() {
            let pk_str = self.pubkey_field.get_text();
            bitcoin::PublicKey::from_str(&pk_str)?
        } else {
            let master = ExtendedPubKey::from_str(&self.xpub_field.get_text())?;

            let derivation = self.derivation_path()?;

            let xpubkey = master.derive_pub(&lnpbp::SECP256K1, &derivation)?;

            self.xpubid_display
                .set_text(&xpubkey.identifier().to_string());
            self.fingerprint_display
                .set_text(&xpubkey.fingerprint().to_string());
            self.derivation_display.set_text(&derivation.to_string());
            self.descriptor_display.set_text(&format!(
                "[{}]{}",
                master.fingerprint(),
                derivation
            ));
            self.xpub_display.set_text(&xpubkey.to_string());

            xpubkey.public_key
        };

        self.uncompressed_display.set_text(
            &bitcoin::PublicKey {
                compressed: false,
                key: pk.key,
            }
            .to_string(),
        );

        let pkc = bitcoin::PublicKey {
            compressed: true,
            key: pk.key,
        };
        self.compressed_display.set_text(&pkc.to_string());
        self.xcoordonly_display.set_text("Not yet supported");

        self.pkh_display
            .set_text(&bitcoin::Address::p2pkh(&pk, network).to_string());
        self.wpkh_display.set_text(
            &bitcoin::Address::p2wpkh(&pkc, network)
                .expect("The key is compressed")
                .to_string(),
        );
        self.wpkh_sh_display.set_text(
            &bitcoin::Address::p2shwpkh(&pkc, network)
                .expect("The key is compressed")
                .to_string(),
        );
        self.taproot_display.set_text("Not yet supported");

        if self.name_field.get_text().is_empty() {
            let err = Error::EmptyName;
            self.name_field.set_icon_from_icon_name(
                gtk::EntryIconPosition::Secondary,
                Some("dialog-error"),
            );
            self.name_field.set_icon_tooltip_text(
                gtk::EntryIconPosition::Secondary,
                Some(&err.to_string()),
            );
            Err(err)?;
        } else {
            self.name_field.set_icon_from_icon_name(
                gtk::EntryIconPosition::Secondary,
                None,
            );
            self.name_field
                .set_icon_tooltip_text(gtk::EntryIconPosition::Secondary, None);
        }

        Ok(None)
    }
}
