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
