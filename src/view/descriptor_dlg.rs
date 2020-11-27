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

use crate::model::{DescriptorParams, Document};
use crate::view::PubkeySelectDlg;

static UI: &'static str = include_str!("../../ui/descriptor.glade");

#[derive(Debug, Display, From, Error)]
#[display(doc_comments)]
/// Errors from processing descriptor data
pub enum Error {
    /// Temporary error
    None,
}

pub struct DescriptorDlg {
    dialog: gtk::Dialog,

    msg_box: gtk::Box,
    msg_label: gtk::Label,
    msg_image: gtk::Image,

    select_pk_btn: gtk::Button,

    save_btn: gtk::Button,
    cancel_btn: gtk::Button,
}

impl DescriptorDlg {
    pub fn load_glade() -> Result<Rc<Self>, glade::Error> {
        let builder = gtk::Builder::from_string(UI);

        let save_btn = builder.get_object("save")?;
        let cancel_btn = builder.get_object("cancel")?;

        let msg_box = builder.get_object("messageBox")?;
        let msg_image = builder.get_object("messageImage")?;
        let msg_label = builder.get_object("messageLabel")?;

        let select_pk_btn: gtk::Button = builder.get_object("selectPubkey")?;

        let me = Rc::new(Self {
            dialog: glade_load!(builder, "descriptorDlg")?,
            msg_box,
            msg_image,
            msg_label,

            select_pk_btn,

            save_btn,
            cancel_btn,
        });

        Ok(me)
    }
}

impl DescriptorDlg {
    pub fn run(
        self: Rc<Self>,
        doc: Rc<RefCell<Document>>,
        on_save: impl Fn(DescriptorParams) + 'static,
        on_cancel: impl Fn() + 'static,
    ) {
        let me = self.clone();

        me.update_ui();

        me.select_pk_btn.connect_clicked(move |_| {
            let pubkey_dlg = PubkeySelectDlg::load_glade().expect("Must load");
            pubkey_dlg.run(doc.clone(), |tracking_account| {}, || {});
        });

        me.cancel_btn
            .connect_clicked(clone!(@weak self as me => move |_| {
                me.dialog.close();
                on_cancel()
            }));

        me.save_btn.connect_clicked(
            clone!(@weak self as me => move |_| match self.descriptor_params() {
                Ok(descriptor_params) => {
                    me.dialog.close();
                    on_save(descriptor_params);
                }
                Err(err) => {
                    me.display_error(err);
                    me.save_btn.set_sensitive(false);
                }
            }),
        );

        me.dialog.run();
        me.dialog.close();
    }

    pub fn descriptor_params(&self) -> Result<DescriptorParams, Error> {
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

    pub fn update_ui(&self) {
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
        Ok(None)
    }
}
