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

use amplify::internet::InetSocketAddr;
use gtk::prelude::*;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::Mutex;

use lnpbp::bp::Chain;
use lnpbp::lnp::{NodeAddr, RemoteNodeAddr};
use lnpbp::strict_encoding::{self, StrictDecode, StrictEncode};
// use rgb::fungible;

use super::{operation, TrackingAccount, UtxoEntry};

const DOC_NAME: &'static str = "Untitled";
lazy_static! {
    static ref DOC_NO: Mutex<u32> = Mutex::new(0);
}

#[derive(Clone, PartialEq, Eq, Debug, Display, From, Error)]
#[display(doc_comments)]
/// Document-specific errors that may happen during file opening, saving and
/// internal consistency validation
pub enum Error {
    /// File data encoding error
    #[display("{0}")]
    #[from]
    DataEncoding(strict_encoding::Error),

    /// I/O error (file etc)
    Io(io::ErrorKind),

    /// Wrong position: no item exists at position {0}
    WrongPosition(usize),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err.kind())
    }
}

#[derive(Debug, Default)]
pub struct Document {
    name: String,
    file: Option<File>,
    profile: Profile,
}

impl Document {
    pub fn new() -> Document {
        *DOC_NO.lock().unwrap() += 1;
        Document {
            name: format!("{}{}", DOC_NAME, *DOC_NO.lock().unwrap()),
            ..Default::default()
        }
    }

    pub fn load(path: PathBuf) -> Result<Document, Error> {
        let file = File::open(path.clone())?;
        let profile = Profile::strict_decode(&file)?;
        Ok(Document {
            file: Some(file),
            name: path
                .file_stem()
                .and_then(OsStr::to_str)
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    *DOC_NO.lock().unwrap() += 1;
                    format!("{}{}", DOC_NAME, *DOC_NO.lock().unwrap())
                }),
            profile,
        })
    }

    pub fn save(&mut self) -> Result<bool, Error> {
        if self.file.is_some() {
            self.save_internal()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn save_as(&mut self, path: PathBuf) -> Result<(), Error> {
        let file = File::create(path)?;
        self.file = Some(file);
        self.save_internal()?;
        Ok(())
    }

    fn save_internal(&mut self) -> Result<(), Error> {
        let file = self
            .file
            .as_mut()
            .expect("Method always called with file initialized");
        file.seek(SeekFrom::Start(0))?;
        file.set_len(0)?;
        self.profile.strict_encode(file)?;
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.file.is_some()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn fill_tracking_store(&self, store: &gtk::ListStore) {
        store.clear();
        self.profile.tracking.iter().for_each(|tracking_account| {
            store.insert_with_values(
                None,
                &[0, 1, 2],
                &[
                    &tracking_account.name(),
                    &tracking_account.details(),
                    &tracking_account.count(),
                ],
            );
        });
    }

    pub fn tracking_account_at(&self, pos: usize) -> Option<TrackingAccount> {
        self.profile.tracking.get(pos).cloned()
    }

    pub fn tracking_account_by_key(
        &self,
        key: &str,
    ) -> Option<TrackingAccount> {
        self.profile
            .tracking
            .iter()
            .find(|a| a.key.to_string() == key)
            .cloned()
    }

    pub fn add_tracking_account(
        &mut self,
        tracking_account: TrackingAccount,
    ) -> Result<bool, Error> {
        self.profile.tracking.push(tracking_account);
        self.save()
    }

    pub fn update_tracking_account(
        &mut self,
        pos: usize,
        tracking_account: TrackingAccount,
    ) -> Result<bool, Error> {
        if self.profile.tracking.len() <= pos {
            Err(Error::WrongPosition(pos))
        } else {
            self.profile.tracking[pos] = tracking_account;
            self.save()
        }
    }

    pub fn remove_tracking_account_at(
        &mut self,
        pos: usize,
    ) -> Result<bool, Error> {
        if self.profile.tracking.len() <= pos {
            Err(Error::WrongPosition(pos))
        } else {
            self.profile.tracking.remove(pos);
            self.save()
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default, StrictEncode, StrictDecode)]
pub struct Profile {
    pub description: Option<String>,
    pub tracking: Vec<TrackingAccount>,
    pub utxo_cache: Vec<UtxoEntry>,
    // pub assets_cache: Vec<fungible::Asset>,
    pub history: Vec<operation::LogEntry>,
    pub settings: Settings,
}

#[derive(Clone, PartialEq, Eq, Debug, Display, StrictEncode, StrictDecode)]
pub enum ChainResolver {
    #[display("bitcoinCore({0})")]
    BitcoinCore(InetSocketAddr),
    #[display("electrum({0})")]
    Electrum(InetSocketAddr),
    #[display("bpNode({0})")]
    BpNode(NodeAddr),
}

impl Default for ChainResolver {
    fn default() -> Self {
        ChainResolver::Electrum(
            "31.14.40.18:60000"
                .parse()
                .expect("Predefined address always parses"),
        )
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default, StrictEncode, StrictDecode)]
pub struct Settings {
    pub chain: Chain,
    pub resolver: ChainResolver,
    pub bifrost: Option<RemoteNodeAddr>,
}
