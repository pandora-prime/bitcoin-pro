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
use std::collections::{BTreeMap, HashSet};
use std::convert::TryFrom;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;

use electrum_client::{Client as ElectrumClient, Error as ElectrumError};
use lnpbp::bitcoin::OutPoint;

use lnpbp::bitcoin::Transaction;
use lnpbp::bp::{Chain, Psbt};
use lnpbp::lnp::{NodeAddr, RemoteNodeAddr};
use lnpbp::rgb::{Consignment, ContractId, Genesis, Schema, SchemaId};
use lnpbp::strict_encoding::{self, StrictDecode, StrictEncode};
use rgb::fungible::{AccountingAmount, Asset};

use super::{operation, DescriptorGenerator, TrackingAccount, UtxoEntry};
use crate::model::DescriptorContent;

/// Equals to first 4 bytes of SHA256("pandoracore:bpro")
/// = dbe2b664ee4e81d3a55d53aeba1915c468927c79a03587ddfc5c3aec483028ab
/// Check with `echo -n "pandoracore:bpro" | shasum -a 256`
const DOC_MAGIC: u32 = 0xdbe2b664;
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

    /// Attempt to add contract that already exits; if you are trying to
    /// update the version please remove older version first
    DuplicatedContract(ContractId),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err.kind())
    }
}

#[derive(Default)]
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
        let mut profile = Profile::strict_decode(&file)?;
        // TODO: Change this to checking document magic number once all docs
        //       will be updated
        profile.magic = DOC_MAGIC;
        let file = OpenOptions::new().write(true).open(path.clone())?;
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
            ..Default::default()
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

    pub fn chain(&self) -> &Chain {
        &self.profile.settings.chain
    }

    pub fn set_chain(&mut self, chain_name: &str) -> Result<bool, Error> {
        self.profile.settings.chain =
            Chain::from_str(chain_name).unwrap_or(Chain::Testnet3);
        self.save()
    }

    pub fn electrum(&self) -> Option<String> {
        if let ChainResolver::Electrum(electrum) =
            self.profile.settings.resolver
        {
            Some(electrum.to_string())
        } else {
            None
        }
    }

    pub fn set_electrum(
        &mut self,
        addr: InetSocketAddr,
    ) -> Result<bool, Error> {
        self.profile.settings.resolver = ChainResolver::Electrum(addr);
        self.save()
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
        tracking_account: &TrackingAccount,
        new_tracking_account: TrackingAccount,
    ) -> Result<bool, Error> {
        if let Some(account) = self
            .profile
            .tracking
            .iter_mut()
            .find(|a| *a == tracking_account)
        {
            *account = new_tracking_account
        }
        self.save()
    }

    pub fn update_tracking_account_at(
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

    pub fn remove_tracking_account(
        &mut self,
        tracking_account: TrackingAccount,
    ) -> Result<bool, Error> {
        self.profile
            .tracking
            .iter()
            .position(|a| *a == tracking_account)
            .map(|i| self.profile.tracking.remove(i));
        self.save()
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

    pub fn fill_descriptor_store(&self, store: &gtk::ListStore) {
        store.clear();
        self.profile
            .descriptors
            .iter()
            .for_each(|descriptor_generator| {
                store.insert_with_values(
                    None,
                    &[0, 1, 2],
                    &[
                        &descriptor_generator.name(),
                        &descriptor_generator.type_name(),
                        &descriptor_generator.descriptor(),
                    ],
                );
            });
    }

    pub fn descriptor_by_generator(
        &self,
        generator_str: &str,
    ) -> Option<DescriptorGenerator> {
        self.profile
            .descriptors
            .iter()
            .find(|g| g.descriptor() == generator_str)
            .cloned()
    }

    pub fn descriptor_by_content(
        &self,
        content: &DescriptorContent,
    ) -> Option<DescriptorGenerator> {
        self.profile
            .descriptors
            .iter()
            .find(|g| &g.content == content)
            .cloned()
    }

    pub fn add_descriptor(
        &mut self,
        descriptor_generator: DescriptorGenerator,
    ) -> Result<bool, Error> {
        self.profile.descriptors.push(descriptor_generator);
        self.save()
    }

    pub fn update_descriptor(
        &mut self,
        descriptor_generator: &DescriptorGenerator,
        new_descriptor_generator: DescriptorGenerator,
    ) -> Result<bool, Error> {
        if let Some(descriptor) = self
            .profile
            .descriptors
            .iter_mut()
            .find(|d| *d == descriptor_generator)
        {
            *descriptor = new_descriptor_generator
        }
        self.save()
    }

    pub fn remove_descriptor(
        &mut self,
        descriptor_generator: DescriptorGenerator,
    ) -> Result<bool, Error> {
        self.profile
            .descriptors
            .iter()
            .position(|d| *d == descriptor_generator)
            .map(|i| self.profile.descriptors.remove(i));
        self.save()
    }

    pub fn fill_utxo_store(
        &self,
        store: &gtk::ListStore,
        filter_by: Option<&DescriptorGenerator>,
    ) {
        store.clear();
        self.profile.utxo_cache.iter().for_each(|utxo| {
            if filter_by
                .map(|generator| utxo.has_match(generator))
                .unwrap_or(true)
            {
                store.insert_with_values(
                    None,
                    &[0, 1, 2, 3],
                    &[
                        &utxo.outpoint.txid.to_string(),
                        &utxo.outpoint.vout,
                        &utxo.amount,
                        &utxo.height,
                    ],
                );
            }
        });
    }

    pub fn update_utxo_set(
        &mut self,
        utxo_set_update: HashSet<UtxoEntry>,
    ) -> Result<bool, Error> {
        self.profile.utxo_cache.extend(utxo_set_update);
        self.save()
    }

    pub fn utxo_by_outpoint(&self, outpoint: OutPoint) -> Option<UtxoEntry> {
        self.profile
            .utxo_cache
            .iter()
            .find(|utxo| utxo.outpoint == outpoint)
            .cloned()
    }

    pub fn remove_utxo(&mut self, utxo: UtxoEntry) -> Result<bool, Error> {
        self.profile.utxo_cache.remove(&utxo);
        self.save()
    }

    pub fn remove_utxo_by_descriptor(
        &mut self,
        descriptor_generator: DescriptorGenerator,
    ) -> Result<bool, Error> {
        self.profile.utxo_cache = self
            .profile
            .utxo_cache
            .iter()
            .filter(|utxo| !utxo.has_match(&descriptor_generator))
            .cloned()
            .collect();
        self.save()
    }

    pub fn is_outpoint_known(&self, outpoint: OutPoint) -> bool {
        self.profile
            .utxo_cache
            .iter()
            .find(|utxo| utxo.outpoint == outpoint)
            .is_some()
    }

    pub fn fill_asset_store(&self, store: &gtk::ListStore) {
        store.clear();
        self.profile.assets.iter().for_each(|(contract_id, _)| {
            self.asset_by_id(*contract_id).map(|(asset, _)| {
                store.insert_with_values(
                    None,
                    &[0, 1, 2, 3, 4, 5, 6, 7],
                    &[
                        &asset.ticker(),
                        &asset.name(),
                        &asset
                            .known_allocations()
                            .iter()
                            .filter(|(outpoint, _)| {
                                self.is_outpoint_known(**outpoint)
                            })
                            .fold(0f64, |sum, (_, allocations)| {
                                sum + AccountingAmount::from_asset_atomic_value(
                                    &asset,
                                    allocations.iter().fold(0u64, |sum, a| {
                                        sum + a.value().value
                                    }),
                                )
                                .accounting_value()
                            }),
                        &asset.supply().known_circulating().accounting_value(),
                        &1,
                        &(asset.known_inflation().len() > 0),
                        &0,
                        &contract_id.to_string(),
                    ],
                );
            });
        });
    }

    pub fn asset_by_id(
        &self,
        asset_id: ContractId,
    ) -> Option<(Asset, &Genesis)> {
        self.profile.assets.get(&asset_id).and_then(|consignment| {
            Asset::try_from(consignment.genesis.clone())
                .ok()
                .map(|asset| (asset, &consignment.genesis))
        })
    }

    pub fn add_asset(
        &mut self,
        consignment: Consignment,
    ) -> Result<bool, Error> {
        let contract_id = consignment.genesis.contract_id();
        if self.profile.assets.contains_key(&contract_id) {
            Err(Error::DuplicatedContract(contract_id))?
        }
        self.profile.assets.insert(contract_id, consignment);
        self.save()
    }

    pub fn remove_asset(
        &mut self,
        contract_id: ContractId,
    ) -> Result<bool, Error> {
        self.profile
            .assets
            .iter()
            .position(|(id, _)| *id == contract_id)
            .map(|i| self.profile.tracking.remove(i));
        self.save()
    }

    pub fn resolver(&self) -> Result<ElectrumClient, ResolverError> {
        if let ChainResolver::Electrum(addr) = self.profile.settings.resolver {
            Ok(ElectrumClient::new(&addr.to_string(), None)?)
        } else {
            Err(ResolverError::ElectrumRequired)
        }
    }
}

#[derive(Clone, PartialEq, Debug, StrictEncode, StrictDecode)]
pub struct Profile {
    pub magic: u32,
    pub version: u16,
    pub description: Option<String>,
    pub tracking: Vec<TrackingAccount>,
    pub descriptors: Vec<DescriptorGenerator>,
    pub utxo_cache: HashSet<UtxoEntry>,
    pub tx_cache: Vec<Transaction>,
    pub psbts: Vec<Psbt>,
    pub schemata: BTreeMap<SchemaId, Schema>,
    pub assets: BTreeMap<ContractId, Consignment>,
    pub nfts: BTreeMap<ContractId, Consignment>,
    pub identities: BTreeMap<ContractId, Consignment>,
    pub auditlogs: BTreeMap<ContractId, Consignment>,
    pub contracts: BTreeMap<ContractId, Consignment>,
    pub history: Vec<operation::LogEntry>,
    pub settings: Settings,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            magic: DOC_MAGIC,
            version: 0,
            description: None,
            tracking: vec![],
            descriptors: vec![],
            utxo_cache: set![],
            tx_cache: vec![],
            psbts: vec![],
            schemata: bmap![],
            assets: bmap![],
            nfts: bmap![],
            identities: bmap![],
            auditlogs: bmap![],
            contracts: bmap![],
            history: vec![],
            settings: Settings::default(),
        }
    }
}

#[derive(Clone, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum ResolverError {
    /// Electrum-specific error
    #[display("{0}")]
    Electrum(String),

    /// The current version supports only Electrum server; please specify
    /// server connection string in document settings
    ElectrumRequired,
}

impl From<ElectrumError> for ResolverError {
    fn from(err: ElectrumError) -> Self {
        ResolverError::Electrum(format!("{:?}", err))
    }
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
            "31.14.40.18:60601"
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
