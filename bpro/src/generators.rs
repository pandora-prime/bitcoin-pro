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

use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;

use amplify::Wrapper;
use bitcoin::secp256k1::{Secp256k1, Verification};
use bitcoin::Script;
use bitcoin_hd::UnhardenedIndex;
use bitcoin_scripts::{ConvertInfo, PubkeyScript};
#[cfg(feature = "serde")]
use serde_with::{As, DisplayFromStr};

use super::{DeriveLockScript, Error, Expanded, Template, Variants};

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[derive(
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[display("{variants}<{template}>")]
pub struct Generator {
    pub template: Template,

    #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
    pub variants: Variants,
}

/// Error parsing descriptor generator: unrecognized string
#[derive(
    Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, Error,
)]
#[display(doc_comments)]
pub struct GeneratorParseError;

impl FromStr for Generator {
    type Err = GeneratorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.trim_end_matches('>').split('<');
        let me = Generator {
            variants: split
                .next()
                .ok_or(GeneratorParseError)?
                .parse()
                .map_err(|_| GeneratorParseError)?,
            template: split
                .next()
                .ok_or(GeneratorParseError)?
                .parse()
                .map_err(|_| GeneratorParseError)?,
        };
        if split.next().is_some() {
            Err(GeneratorParseError)
        } else {
            Ok(me)
        }
    }
}

impl Generator {
    pub fn descriptors<C: Verification>(
        &self,
        ctx: &Secp256k1<C>,
        index: UnhardenedIndex,
    ) -> Result<HashMap<ConvertInfo, Expanded>, Error> {
        let mut descriptors = HashMap::with_capacity(5);
        let single = if let Template::SingleSig(_) = self.template {
            Some(
                self.template
                    .try_derive_public_key(ctx, index)
                    .expect("Can't fail"),
            )
        } else {
            None
        };
        if self.variants.bare {
            let d = if let Some(pk) = single {
                Expanded::Pk(pk)
            } else {
                Expanded::Bare(
                    self.template
                        .derive_lock_script(ctx, index, ConvertInfo::Bare)?
                        .into_inner()
                        .into(),
                )
            };
            descriptors.insert(ConvertInfo::Bare, d);
        }
        if self.variants.hashed {
            let d = if let Some(pk) = single {
                Expanded::Pkh(pk)
            } else {
                Expanded::Sh(
                    self.template
                        .derive_lock_script(ctx, index, ConvertInfo::Hashed)?
                        .into(),
                )
            };
            descriptors.insert(ConvertInfo::Hashed, d);
        }
        if self.variants.nested {
            let d = if let Some(pk) = single {
                Expanded::ShWpkh(pk)
            } else {
                Expanded::ShWsh(
                    self.template
                        .derive_lock_script(ctx, index, ConvertInfo::NestedV0)?
                        .into(),
                )
            };
            descriptors.insert(ConvertInfo::NestedV0, d);
        }
        if self.variants.segwit {
            let d = if let Some(pk) = single {
                Expanded::Wpkh(pk)
            } else {
                Expanded::Wsh(
                    self.template
                        .derive_lock_script(ctx, index, ConvertInfo::SegWitV0)?
                        .into(),
                )
            };
            descriptors.insert(ConvertInfo::SegWitV0, d);
        }
        /* TODO #15: Enable once Taproot will go live
        if self.variants.taproot {
            scripts.push(content.taproot());
        }
         */
        Ok(descriptors)
    }

    #[inline]
    pub fn pubkey_scripts<C: Verification>(
        &self,
        ctx: &Secp256k1<C>,
        index: UnhardenedIndex,
    ) -> Result<HashMap<ConvertInfo, Script>, Error> {
        self.descriptors(ctx, index)?
            .into_iter()
            .map(|(cat, descr)| {
                Ok((cat, PubkeyScript::try_from(descr)?.into()))
            })
            .collect()
    }
}
