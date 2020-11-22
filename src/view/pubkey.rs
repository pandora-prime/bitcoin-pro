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

//#[glade_load="../ui/asset_issue.glade"]
pub struct PubkeyDlg {
    //#[glade_id="assetIssue"]
    dialog: gtk::Dialog,
}

impl glade::View for PubkeyDlg {
    fn load_glade() -> Result<Rc<RefCell<Self>>, glade::Error> {
        let builder = gtk::Builder::from_string(UI);
        Ok(Rc::new(RefCell::new(Self {
            dialog: glade_load!(builder, "pubkeyDlg")?,
        })))
    }
}

impl PubkeyDlg {
    pub fn run(&self) {
        self.dialog.run();
        self.dialog.hide();
    }
}
