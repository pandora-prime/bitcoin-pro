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

static UI: &str = include_str!("../view/file_save.glade");

pub struct SaveDlg {
    dialog: gtk::FileChooserDialog,
    save_btn: gtk::Button,
    cancel_btn: gtk::Button,
}

impl SaveDlg {
    pub fn load_glade() -> Option<Rc<Self>> {
        let builder = gtk::Builder::from_string(UI);

        let save_btn = builder.get_object("save")?;
        let cancel_btn = builder.get_object("cancel")?;
        let dialog = builder.get_object("saveDlg")?;

        Some(Rc::new(SaveDlg {
            dialog,
            save_btn,
            cancel_btn,
        }))
    }

    pub fn run(
        self: Rc<Self>,
        name: String,
        on_save: impl Fn(PathBuf) + 'static,
        on_cancel: impl Fn() + 'static,
    ) {
        let me = self.clone();

        me.dialog.set_current_name(name.clone());

        me.cancel_btn
            .connect_clicked(clone!(@weak self as me => move |_| {
                me.dialog.hide();
                on_cancel()
            }));

        me.save_btn
            .connect_clicked(clone!(@weak self as me, @strong name => move |_| {
                if let Some(mut path) = me.dialog.get_current_folder() {
                    me.dialog.hide();
                    path.push(me.dialog.get_current_name().unwrap_or_else(|| name.clone().into()).as_str());
                    on_save(path);
                }
            }));

        me.dialog.run();
        me.dialog.hide();
    }
}
