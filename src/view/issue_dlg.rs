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
use std::rc::Rc;

use crate::model::AssetGenesis;

static UI: &'static str = include_str!("../../ui/asset_create.glade");

#[derive(Debug, Display, From, Error)]
#[display(doc_comments)]
/// Errors from processing asset genesis data
pub enum Error {
    /// Temporary error
    None,
}

pub struct IssueDlg {
    dialog: gtk::Dialog,
    msg_box: gtk::Box,
    msg_label: gtk::Label,
    msg_image: gtk::Image,

    create_btn: gtk::Button,
    cancel_btn: gtk::Button,
}

impl IssueDlg {
    pub fn load_glade() -> Result<Rc<Self>, glade::Error> {
        let builder = gtk::Builder::from_string(UI);

        let create_btn = builder.get_object("create")?;
        let cancel_btn = builder.get_object("cancel")?;

        let msg_box = builder.get_object("messageBox")?;
        let msg_image = builder.get_object("messageImage")?;
        let msg_label = builder.get_object("messageLabel")?;

        let me = Rc::new(Self {
            dialog: glade_load!(builder, "assetCreateDlg")?,
            msg_box,
            msg_image,
            msg_label,

            create_btn,
            cancel_btn,
        });

        Ok(me)
    }
}

impl IssueDlg {
    pub fn run(
        self: Rc<Self>,
        on_issue: impl Fn(AssetGenesis) + 'static,
        on_cancel: impl Fn() + 'static,
    ) {
        let me = self.clone();

        me.update_ui();

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
