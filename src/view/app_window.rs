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

use glade::View;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use super::PubkeyDlg;
use crate::model::Document;
use crate::view::SaveDlg;

static UI: &'static str = include_str!("../../ui/main.glade");

#[derive(Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum Error {
    /// Glade error: {0}
    #[from]
    GladeError(glade::Error),
}

pub struct AppWindow {
    window: gtk::ApplicationWindow,
    pubkey_tree: gtk::TreeView,
    pubkey_store: gtk::TreeStore,
    header_bar: gtk::HeaderBar,
}

impl View for AppWindow {
    fn load_glade() -> Result<Rc<RefCell<Self>>, glade::Error> {
        let doc = Rc::new(RefCell::new(Document::new()));

        let builder = gtk::Builder::from_string(UI);

        let pubkey_tree: gtk::TreeView = builder.get_object("pubkeyTree")?;
        let pubkey_store = builder.get_object("pubkeyStore")?;
        let header_bar: gtk::HeaderBar = builder.get_object("headerBar")?;
        pubkey_tree.set_model(Some(&pubkey_store));
        pubkey_tree.expand_all();

        header_bar.set_subtitle(Some(&doc.borrow().name()));

        let me = Rc::new(RefCell::new(Self {
            window: glade_load!(builder, "appWindow")?,
            pubkey_tree,
            pubkey_store,
            header_bar,
        }));

        let tb: gtk::ToolButton = builder.get_object("pubkeyAdd")?;
        tb.connect_clicked(clone!(@weak me, @strong doc => move |_| {
            let pubkey_dlg = PubkeyDlg::load_glade().expect("Must load");
            pubkey_dlg.run(clone!(@weak me, @strong doc =>
                move |tracking_account| {
                    let me = me.borrow();
                    me.pubkey_store.insert_with_values(
                        None,
                        None,
                        &[0, 1, 2],
                        &[
                            &tracking_account.name(),
                            &tracking_account.details(),
                            &tracking_account.count(),
                        ],
                    );
                    let _ = doc.borrow_mut().add_tracking_account(tracking_account);
                }),
                || {},
            );
        }));

        /*
        let tb: gtk::Button = builder.get_object("open")?;
        tb.connect_clicked(clone!(@strong doc => move |_| {
            let open_dlg = OpenDlg::load_glade().expect("Must load");
            open_dlg.run(clone!(@strong doc => move |path| {
                let _ = doc.borrow().open(path);
            }), || {})
        }));
         */

        let tb: gtk::Button = builder.get_object("save")?;
        tb.connect_clicked(clone!(@strong doc, @weak tb => move |_| {
            let save_dlg = SaveDlg::load_glade().expect("Must load");
            let name = doc.borrow().name();
            save_dlg.run(name, clone!(@strong doc, @weak tb => move |path| {
                let mut path = path;
                path.set_extension("bpro");
                let _ = doc.borrow_mut().save_as(path).and_then(|_| {
                    tb.set_sensitive(false);
                    Ok(())
                });
            }), || {})
        }));

        Ok(me)
    }
}

impl AppWindow {
    pub fn new() -> Result<Rc<RefCell<Self>>, Error> {
        let me = Self::load_glade()?;
        Ok(me)
    }

    pub fn run(&self) {
        self.update_ui();
        self.window.show_all();
        gtk::main();
    }

    pub fn update_ui(&self) {}
}
