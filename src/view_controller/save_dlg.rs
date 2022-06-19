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
