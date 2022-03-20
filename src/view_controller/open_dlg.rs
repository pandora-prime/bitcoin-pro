// Bitcoin Pro: Professional bitcoin accounts & assets management
// Written in 2020-2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use gtk::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;

static UI: &str = include_str!("../view/file_open.glade");

pub struct OpenDlg {
    dialog: gtk::FileChooserDialog,
    open_btn: gtk::Button,
    cancel_btn: gtk::Button,
}

impl OpenDlg {
    pub fn load_glade() -> Option<Rc<Self>> {
        let builder = gtk::Builder::from_string(UI);

        let open_btn = builder.get_object("open")?;
        let cancel_btn = builder.get_object("cancel")?;
        let dialog = builder.get_object("openDlg")?;

        Some(Rc::new(OpenDlg {
            dialog,
            open_btn,
            cancel_btn,
        }))
    }

    pub fn run(
        self: Rc<Self>,
        on_open: impl Fn(PathBuf) + 'static,
        on_cancel: impl Fn() + 'static,
    ) {
        let me = self.clone();

        me.cancel_btn
            .connect_clicked(clone!(@weak self as me => move |_| {
                me.dialog.hide();
                on_cancel()
            }));

        me.open_btn
            .connect_clicked(clone!(@weak self as me => move |_| {
                if let Some(path) = me.dialog.get_filename() {
                    me.dialog.hide();
                    on_open(path);
                }
            }));

        me.dialog.run();
        me.dialog.hide();
    }
}
