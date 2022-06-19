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

use gtk::prelude::GtkListStoreExtManual;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::DerefMut;
use std::rc::Rc;

use bitcoin::Script;
use electrum_client::{
    Client as ElectrumClient, ElectrumApi, Error as ElectrumError,
};
use wallet::descriptors;
use wallet::hd::{SegmentIndexes, UnhardenedIndex};

use crate::model::{DescriptorAccount, UtxoEntry};
use crate::util::resolver_mode::ResolverModeType;

#[derive(Clone, PartialEq, Eq, Debug, Display, From, Error)]
#[display(doc_comments)]
pub enum Error {
    /// Electrum error
    #[display("{0}")]
    #[from]
    Electrum(String),

    /// The actual value of the used index corresponds to a hardened index,
    /// which can't be used in the current context
    HardenedIndex,

    /// Unable to generate key with index {0} for descriptor {1}: {2}
    Descriptor(u32, String, descriptors::Error),
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
        account: DescriptorAccount,
        utxo_set: Rc<RefCell<HashSet<UtxoEntry>>>,
        uxto_store: Option<&gtk::ListStore>,
    ) -> Result<usize, Error> {
        struct LookupItem<'a> {
            pub script_pubkey: Script,
            pub descriptor_type: descriptors::Category,
            pub descriptor_content: &'a descriptors::Template,
            pub derivation_index: u32,
        }

        let mut total_found = 0usize;
        let mut lookup_iter = lookup_type.into_iter();
        loop {
            let mut lookup: Vec<LookupItem> = Vec::with_capacity(
                lookup_type.count() as usize
                    * account.pubkey_scripts_count() as usize,
            );
            for offset in lookup_iter.by_ref() {
                let scripts = account
                    .pubkey_scripts(
                        UnhardenedIndex::from_index(offset)
                            .map_err(|_| Error::HardenedIndex)?,
                    )
                    .map_err(|err| {
                        Error::Descriptor(offset, account.descriptor(), err)
                    })?;
                lookup.extend(scripts.into_iter().map(
                    |(descriptor_type, script_pubkey)| LookupItem {
                        script_pubkey,
                        descriptor_type,
                        descriptor_content: &account.generator.template,
                        derivation_index: offset,
                    },
                ));
            }
            let mut found = 0usize;
            let request: Vec<_> = lookup
                .iter()
                .map(|item| item.script_pubkey.clone())
                .collect();
            println!("Requesting lookup for: {:#?}", request);
            let response =
                resolver.batch_script_list_unspent(request.iter())?;
            println!("Response:\n{:#?}", response);
            for utxo in
                response.into_iter().zip(lookup).flat_map(|(list, item)| {
                    list.into_iter().map(move |res| {
                        UtxoEntry::with(
                            &res,
                            item.descriptor_content.clone(),
                            item.descriptor_type,
                            item.derivation_index,
                        )
                    })
                })
            {
                found += 1;
                if utxo_set.borrow_mut().deref_mut().insert(utxo.clone()) {
                    if let Some(utxo_store) = uxto_store {
                        utxo_store.insert_with_values(
                            None,
                            &[
                                (0, &utxo.outpoint.txid.to_string()),
                                (1, &utxo.outpoint.vout),
                                (2, &utxo.amount),
                                (3, &utxo.height),
                            ],
                        );
                    }
                }
            }
            total_found += found;
            if !lookup_type.is_while() || found == 0 {
                break;
            }
        }
        Ok(total_found)
    }
}
