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
use std::path::PathBuf;
use std::rc::Rc;

use crate::model::Document;
use crate::view::{DescriptorDlg, IssueDlg, PubkeyDlg, SaveDlg};

static UI: &'static str = include_str!("../../ui/main.glade");

#[derive(Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum Error {
    /// Glade error: {0}
    #[from]
    GladeError(glade::Error),

    /// Document-based error
    #[from]
    #[display("{0}")]
    Document(crate::model::Error),
}

pub struct AppWindow {
    window: gtk::ApplicationWindow,
    pubkey_tree: gtk::TreeView,
    pubkey_store: gtk::ListStore,
    header_bar: gtk::HeaderBar,
    new_btn: gtk::Button,
    open_btn: gtk::Button,
}

impl AppWindow {
    fn load_glade(
        doc: Option<Document>,
    ) -> Result<Rc<RefCell<Self>>, glade::Error> {
        let mut needs_save = true;
        let doc = Rc::new(RefCell::new(if let Some(doc) = doc {
            needs_save = false;
            doc
        } else {
            Document::new()
        }));

        let builder = gtk::Builder::from_string(UI);

        let new_btn: gtk::Button = builder.get_object("new")?;
        let open_btn: gtk::Button = builder.get_object("open")?;
        let pubkey_tree: gtk::TreeView = builder.get_object("pubkeyTree")?;
        let pubkey_store = builder.get_object("pubkeyStore")?;
        let header_bar: gtk::HeaderBar = builder.get_object("headerBar")?;

        doc.borrow().fill_tracking_store(&pubkey_store);
        pubkey_tree.set_model(Some(&pubkey_store));
        pubkey_tree.expand_all();

        header_bar.set_subtitle(Some(&doc.borrow().name()));

        let me = Rc::new(RefCell::new(Self {
            window: glade_load!(builder, "appWindow")?,
            pubkey_tree,
            pubkey_store,
            header_bar,
            new_btn,
            open_btn,
        }));

        let tb: gtk::ToolButton = builder.get_object("pubkeyAdd")?;
        tb.connect_clicked(clone!(@weak me, @strong doc => move |_| {
            let pubkey_dlg = PubkeyDlg::load_glade().expect("Must load");
            pubkey_dlg.run(clone!(@weak me, @strong doc =>
                move |tracking_account| {
                    let me = me.borrow();
                    me.pubkey_store.insert_with_values(
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

        let tb: gtk::ToolButton = builder.get_object("descriptorAdd")?;
        tb.connect_clicked(clone!(@weak me, @strong doc => move |_| {
            let descriptor_dlg = DescriptorDlg::load_glade().expect("Must load");
            descriptor_dlg.run(doc.clone(), clone!(@weak me, @strong doc =>
                move |descriptor_params| {
                    let me = me.borrow();
                    /* TODO: Perform assst creation
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
                     */
                }),
                || {},
            );
        }));

        let tb: gtk::ToolButton = builder.get_object("assetCreate")?;
        tb.connect_clicked(clone!(@weak me, @strong doc => move |_| {
            let issue_dlg = IssueDlg::load_glade().expect("Must load");
            issue_dlg.run(clone!(@weak me, @strong doc =>
                move |asset_genesis| {
                    let me = me.borrow();
                    /* TODO: Perform assst creation
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
                     */
                }),
                || {},
            );
        }));

        let tb: gtk::Button = builder.get_object("save")?;
        tb.set_sensitive(needs_save);
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
    pub fn new(path: Option<PathBuf>) -> Result<Rc<RefCell<Self>>, Error> {
        let doc = if let Some(path) = path {
            Some(Document::load(path)?)
        } else {
            None
        };
        let me = Self::load_glade(doc)?;
        Ok(me)
    }

    pub fn run(
        &self,
        on_open: impl Fn() + 'static,
        on_new: impl Fn() + 'static,
    ) {
        self.update_ui();

        self.new_btn.connect_clicked(move |_| on_new());
        self.open_btn.connect_clicked(move |_| on_open());

        self.window.show_all();
        gtk::main();
    }

    pub fn update_ui(&self) {}
}
