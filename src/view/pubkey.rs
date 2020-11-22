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

    name_field: gtk::Entry,
    pubkey_field: gtk::Entry,
    xpub_field: gtk::Entry,

    sk_radio: gtk::RadioButton,
    hd_radio: gtk::RadioButton,

    bip44_radio: gtk::RadioButton,
    custom_radio: gtk::RadioButton,

    pk_cache: String,
}

impl glade::View for PubkeyDlg {
    fn load_glade() -> Result<Rc<RefCell<Self>>, glade::Error> {
        let builder = gtk::Builder::from_string(UI);

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
        let me = Rc::new(RefCell::new(Self {
            dialog: glade_load!(builder, "pubkeyDlg")?,
            name_field,
            pubkey_field,
            xpub_field,
            sk_radio,
            hd_radio,
            bip44_radio,
            custom_radio,
            pk_cache: none!(),
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
    }
}
