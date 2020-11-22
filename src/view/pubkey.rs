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
use std::str::FromStr;

use lnpbp::{bitcoin, secp256k1};

static UI: &'static str = include_str!("../../ui/pubkey.glade");

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Display)]
#[display(Debug)]
pub enum PkType {
    Single,
    Hd,
}

//#[glade_load="../ui/asset_issue.glade"]
pub struct PubkeyDlg {
    //#[glade_id="assetIssue"]
    dialog: gtk::Dialog,
    msg_box: gtk::Box,
    msg_label: gtk::Label,
    msg_image: gtk::Image,

    name_field: gtk::Entry,
    pubkey_field: gtk::Entry,
    xpub_field: gtk::Entry,

    sk_radio: gtk::RadioButton,
    hd_radio: gtk::RadioButton,

    bip44_radio: gtk::RadioButton,
    custom_radio: gtk::RadioButton,

    network_combo: gtk::ComboBox,

    uncompressed_display: gtk::Entry,
    compressed_display: gtk::Entry,
    xcoordonly_display: gtk::Entry,

    pkh_display: gtk::Entry,
    wpkh_display: gtk::Entry,
    wpkh_sh_display: gtk::Entry,
    taproot_display: gtk::Entry,
}

impl glade::View for PubkeyDlg {
    fn load_glade() -> Result<Rc<RefCell<Self>>, glade::Error> {
        let builder = gtk::Builder::from_string(UI);

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

        let network_combo = builder
            .get_object("blockchainCombo")
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

        let me = Rc::new(RefCell::new(Self {
            dialog: glade_load!(builder, "pubkeyDlg")?,
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
            network_combo,
            uncompressed_display,
            compressed_display,
            xcoordonly_display,
            pkh_display,
            wpkh_display,
            wpkh_sh_display,
            taproot_display,
        }));

        me.as_ref().borrow().pubkey_field.connect_changed(
            clone!(@weak me => move |_| {
                let me = me.borrow();
                me.set_key_type(PkType::Single, true)
            }),
        );

        me.as_ref().borrow().xpub_field.connect_changed(
            clone!(@weak me => move |_| {
                let me = me.borrow();
                me.set_key_type(PkType::Hd, true)
            }),
        );

        me.as_ref().borrow().sk_radio.connect_toggled(
            clone!(@weak me => move |_| {
                let me = me.borrow();
                me.set_key_type(PkType::Single, false)
            }),
        );

        me.as_ref().borrow().hd_radio.connect_toggled(
            clone!(@weak me => move |_| {
                let me = me.borrow();
                me.set_key_type(PkType::Hd, false)
            }),
        );

        me.as_ref().borrow().network_combo.connect_changed(
            clone!(@weak me => move |_| {
                let me = me.borrow();
                me.update_ui()
            }),
        );

        Ok(me)
    }
}

impl PubkeyDlg {
    pub fn run(&self) {
        self.set_key_type(
            if self.sk_radio.get_active() {
                PkType::Single
            } else {
                PkType::Hd
            },
            false,
        );
        self.dialog.run();
        self.dialog.hide();
    }

    pub fn set_key_type(&self, pk_type: PkType, activate: bool) {
        self.pubkey_field.set_sensitive(pk_type == PkType::Single);
        self.xpub_field.set_sensitive(pk_type == PkType::Hd);
        if activate {
            self.sk_radio.set_active(pk_type == PkType::Single);
            self.hd_radio.set_active(pk_type == PkType::Hd);
        }
        self.update_ui();
    }

    pub fn update_ui(&self) {
        match self.update_ui_internal() {
            Ok(_) => {
                self.msg_box.set_visible(false);
            }
            Err(err) => {
                self.msg_label.set_text(&err);
                self.msg_box.set_visible(true);
            }
        }
    }

    pub fn update_ui_internal(&self) -> Result<(), String> {
        let network = match self.network_combo.get_active() {
            Some(0) => bitcoin::Network::Bitcoin,
            Some(1) => bitcoin::Network::Testnet,
            Some(2) => bitcoin::Network::Testnet,
            None => Err("You need to specify blockchain type")?,
            _ => Err("Unsupported blockchain")?,
        };

        let pk_type = if self.sk_radio.get_active() {
            let pk_str = self.pubkey_field.get_text();
            let pk = secp256k1::PublicKey::from_str(&pk_str)
                .map_err(|err| err.to_string())?;

            self.uncompressed_display.set_text(
                &bitcoin::PublicKey {
                    compressed: false,
                    key: pk,
                }
                .to_string(),
            );

            let pk = bitcoin::PublicKey {
                compressed: true,
                key: pk,
            };
            self.compressed_display.set_text(&pk.to_string());
            self.xcoordonly_display.set_text("Not yet supported");

            self.pkh_display
                .set_text(&bitcoin::Address::p2pkh(&pk, network).to_string());
            self.wpkh_display.set_text(
                &bitcoin::Address::p2wpkh(&pk, network)
                    .expect("The key is compressed")
                    .to_string(),
            );
            self.wpkh_sh_display.set_text(
                &bitcoin::Address::p2shwpkh(&pk, network)
                    .expect("The key is compressed")
                    .to_string(),
            );
            self.taproot_display.set_text("Not yet supported");

            PkType::Single
        } else {
            PkType::Hd
        };

        Ok(())
    }
}
