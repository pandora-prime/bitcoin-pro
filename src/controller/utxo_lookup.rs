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

use gtk::prelude::GtkListStoreExtManual;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::DerefMut;
use std::rc::Rc;

use electrum_client::{
    Client as ElectrumClient, ElectrumApi, Error as ElectrumError,
};

use crate::model::{DescriptorError, DescriptorGenerator, UtxoEntry};
use crate::util::resolver_mode::ResolverModeType;

#[derive(Clone, PartialEq, Eq, Debug, Display, From, Error)]
#[display(doc_comments)]
pub enum Error {
    /// Electrum error
    #[display("{0}")]
    #[from]
    Electrum(String),

    /// Unable to generate key with index {0} for descriptor {1}: {2}
    Descriptor(u32, String, DescriptorError),
}

impl From<ElectrumError> for Error {
    fn from(err: ElectrumError) -> Self {
        Error::Electrum(format!("{:?}", err))
    }
}

pub trait UtxoLookup {
    fn utxo_lookup(
        &self,
        resolver: ElectrumClient,
        lookup_type: ResolverModeType,
        generator: DescriptorGenerator,
        utxo_set: Rc<RefCell<HashSet<UtxoEntry>>>,
        uxto_store: Option<&gtk::ListStore>,
    ) -> Result<usize, Error> {
        let mut total_found = 0usize;
        loop {
            let mut pubkeys = Vec::with_capacity(
                lookup_type.count() as usize
                    * generator.script_pubket_count() as usize,
            );
            for offset in lookup_type {
                pubkeys.extend(generator.script_pubkey(offset).map_err(
                    |err| {
                        Error::Descriptor(offset, generator.descriptor(), err)
                    },
                )?);
            }
            let mut found = 0usize;
            for utxo in resolver
                .batch_script_list_unspent(pubkeys.iter())?
                .into_iter()
                .flatten()
                .map(UtxoEntry::from)
            {
                found += 1;
                if utxo_set.borrow_mut().deref_mut().insert(utxo.clone()) {
                    if let Some(utxo_store) = uxto_store {
                        utxo_store.insert_with_values(
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
                }
            }
            total_found += found;
            if lookup_type.is_while() || found == 0 {
                break;
            }
        }
        Ok(total_found)
    }
}
