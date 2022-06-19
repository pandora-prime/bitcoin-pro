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

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use bitcoin::secp256k1::{self, Secp256k1, Verification};
use bitcoin::util::bip32::{DerivationPath, Fingerprint};
use bitcoin_hd::{
    ComponentsParseError, DerivationComponents, DerivePublicKey,
    UnhardenedIndex,
};
use bitcoin_scripts::convert::ConvertInfo;
use bitcoin_scripts::LockScript;
use miniscript::descriptor::DescriptorSinglePub;
use miniscript::{Miniscript, MiniscriptKey, ToPublicKey, TranslatePk2};
#[cfg(feature = "serde")]
use serde_with::{As, DisplayFromStr};

use super::{DeriveLockScript, Error, ScriptConstruction, ScriptSource};

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename = "lowercase", untagged)
)]
#[derive(
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Debug,
    StrictEncode,
    StrictDecode,
)]
#[non_exhaustive]
pub enum SingleSig {
    /// Single known public key
    #[cfg_attr(feature = "serde", serde(skip))]
    Pubkey(
        // TODO: Update serde serializer once miniscript will have
        //       Display/FromStr
        // #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
        DescriptorSinglePub,
    ),

    /// Public key range with deterministic derivation that can be derived
    /// from a known extended public key without private key
    #[cfg_attr(feature = "serde", serde(rename = "xpub"))]
    XPubDerivable(
        #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
        DerivationComponents,
    ),
}

impl Display for SingleSig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SingleSig::Pubkey(pk) => {
                if let Some((fp, path)) = &pk.origin {
                    let path = path.to_string().replace("m/", "");
                    write!(f, "[{}]/{}/", fp, path)?;
                }
                Display::fmt(&pk.key, f)
            }
            SingleSig::XPubDerivable(xpub) => Display::fmt(xpub, f),
        }
    }
}

impl SingleSig {
    pub fn count(&self) -> u32 {
        match self {
            SingleSig::Pubkey(_) => 1,
            SingleSig::XPubDerivable(ref components) => components.count(),
        }
    }
}

impl DerivePublicKey for SingleSig {
    fn derive_public_key<C: Verification>(
        &self,
        ctx: &Secp256k1<C>,
        child_index: UnhardenedIndex,
    ) -> secp256k1::PublicKey {
        match self {
            SingleSig::Pubkey(ref pkd) => pkd.key.to_public_key().key,
            SingleSig::XPubDerivable(ref dc) => {
                dc.derive_public_key(ctx, child_index)
            }
        }
    }
}

impl MiniscriptKey for SingleSig {
    type Hash = Self;

    fn to_pubkeyhash(&self) -> Self::Hash {
        self.clone()
    }
}

impl FromStr for SingleSig {
    type Err = ComponentsParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(parts) = SingleSigDescriptorParts::from_str(s) {
            let origin = if let Some(fp) = parts.fingerprint {
                let fp = fp
                    .parse::<Fingerprint>()
                    .map_err(|err| ComponentsParseError(err.to_string()))?;
                let deriv = format!(
                    "m/{}",
                    parts
                        .derivation
                        .expect("wrong build-in pubkey parsing syntax")
                )
                .parse::<DerivationPath>()
                .map_err(|err| ComponentsParseError(err.to_string()))?;
                Some((fp, deriv))
            } else {
                None
            };
            let key = bitcoin::PublicKey::from_str(parts.pubkey)
                .map_err(|err| ComponentsParseError(err.to_string()))?;
            Ok(SingleSig::Pubkey(DescriptorSinglePub { origin, key }))
        } else {
            Ok(SingleSig::XPubDerivable(DerivationComponents::from_str(s)?))
        }
    }
}

/// Components of a single sig descriptor.
/// Only used to split a descriptor string into its different parts.
#[derive(Debug, PartialEq, Eq)]
struct SingleSigDescriptorParts<'a> {
    /// Fingerprint of the key
    fingerprint: Option<&'a str>,
    /// Derivation path starting with a digit
    derivation: Option<&'a str>,
    /// Pubkey is either compressed (66 characters) or uncompressed (130
    /// characters)
    pubkey: &'a str,
}

impl<'a> SingleSigDescriptorParts<'a> {
    /// Attempts to split descriptor `s` into its [SingleSigDescriptorParts].
    /// `None` will be returned if `s` is invalid.
    fn from_str(s: &'a str) -> Option<SingleSigDescriptorParts> {
        // Should yield a key which contains a public key + optional fingerprint
        // and type expressions. Reverse splitted because key succeeds
        // type expressions and we need the key first. Remove empty
        // strings of split result caused by splitting parenthesis at the
        // string's end
        let mut key_and_types =
            s.rsplit(&['(', ')'][..]).filter(|s| !s.is_empty());

        if let Some(key) = key_and_types.next() {
            let mut pubkey_and_fingerprint = key.rsplit(&['[', ']'][..]);

            // Checking if public key is present and valid
            let pubkey = if let Some(pubkey) = pubkey_and_fingerprint.next() {
                // Public key needs to start with 0, be either 66 or 130 chars
                // long AND all chars have to be hex
                if (pubkey.len() == 66 || pubkey.len() == 130)
                    && pubkey.chars().all(|c: char| c.is_ascii_hexdigit())
                    && pubkey.starts_with('0')
                {
                    pubkey
                } else {
                    // Public key is invalid
                    return None;
                }
            } else {
                // Public key is not present
                return None;
            };

            // Checking if fingerprint and derivation are present and valid
            let (fingerprint, derivation) =
                if let Some(fingerprint) = pubkey_and_fingerprint.next() {
                    // Split fingerprint into fingerprint and derivation path at
                    // the first '/'
                    if let Some((fingerprint, derivation)) =
                        fingerprint.split_once('/')
                    {
                        // Fingerprint needs to be hex and exactly 8 chars long
                        let is_invalid_fingerprint = fingerprint.len() != 8
                            || fingerprint
                                .chars()
                                .any(|c: char| !c.is_ascii_hexdigit());
                        // Derivation path starts with digit and only contains
                        // digits and '/', 'h' or '
                        let is_invalid_derivation = derivation.is_empty()
                            || derivation
                                .starts_with(|c: char| !c.is_ascii_digit())
                            || derivation.chars().any(|c| {
                                !c.is_ascii_digit() && !"/h'".contains(c)
                            });
                        if is_invalid_fingerprint || is_invalid_derivation {
                            (None, None)
                        } else {
                            // Fingerprint and derivation are ok
                            (Some(fingerprint), Some(derivation))
                        }
                    } else {
                        // Fingerprint couldn't be splitted into fingerprint and
                        // derivation path
                        (None, None)
                    }
                } else {
                    // Input s does not contain a fingerprint
                    (None, None)
                };

            // Return parts of successfully splitted input s
            return Some(Self {
                fingerprint,
                derivation,
                pubkey,
            });
        }

        // Input s could not be splitted into key and expression types
        None
    }
}

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
    StrictEncode,
    StrictDecode,
)]
pub struct MultiSig {
    pub threshold: Option<u8>,

    #[cfg_attr(feature = "serde", serde(with = "As::<Vec<DisplayFromStr>>"))]
    pub pubkeys: Vec<SingleSig>,

    pub reorder: bool,
}

impl Display for MultiSig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "multi({},", self.threshold())?;
        f.write_str(
            &self
                .pubkeys
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        )?;
        f.write_str(")")
    }
}

impl MultiSig {
    pub fn threshold(&self) -> usize {
        self.threshold
            .map(|t| t as usize)
            .unwrap_or(self.pubkeys.len())
    }

    pub fn derive_public_keys<C: Verification>(
        &self,
        ctx: &Secp256k1<C>,
        child_index: UnhardenedIndex,
    ) -> Vec<bitcoin::PublicKey> {
        let mut set = self
            .pubkeys
            .iter()
            .map(|key| key.derive_public_key(ctx, child_index))
            .map(bitcoin::PublicKey::new)
            .collect::<Vec<_>>();
        if self.reorder {
            set.sort();
        }
        set
    }
}

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
    StrictEncode,
    StrictDecode,
)]
pub struct MuSigBranched {
    #[cfg_attr(feature = "serde", serde(with = "As::<Vec<DisplayFromStr>>"))]
    pub extra_keys: Vec<SingleSig>,

    pub tapscript: ScriptConstruction,

    pub source: Option<String>,
}

impl Display for MuSigBranched {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{};", self.tapscript)?;
        f.write_str(
            &self
                .extra_keys
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        )
    }
}

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename = "lowercase")
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
#[display(inner)]
#[non_exhaustive]
#[allow(clippy::large_enum_variant)]
pub enum Template {
    SingleSig(
        #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
        SingleSig,
    ),

    MultiSig(MultiSig),

    Scripted(ScriptSource),

    #[cfg_attr(feature = "serde", serde(rename = "musig"))]
    MuSigBranched(MuSigBranched),
}

// TODO: Provide full implementation
impl FromStr for Template {
    type Err = ComponentsParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Template::SingleSig(SingleSig::from_str(s)?))
    }
}

impl Template {
    #[inline]
    pub fn is_singlesig(&self) -> bool {
        matches!(self, Template::SingleSig(_))
    }

    pub fn try_derive_public_key<C: Verification>(
        &self,
        ctx: &Secp256k1<C>,
        child_index: UnhardenedIndex,
    ) -> Option<bitcoin::PublicKey> {
        match self {
            Template::SingleSig(key) => Some(bitcoin::PublicKey::new(
                key.derive_public_key(ctx, child_index),
            )),
            _ => None,
        }
    }
}

impl DeriveLockScript for MultiSig {
    fn derive_lock_script<C: Verification>(
        &self,
        ctx: &Secp256k1<C>,
        child_index: UnhardenedIndex,
        descr_category: ConvertInfo,
    ) -> Result<LockScript, Error> {
        match descr_category {
            ConvertInfo::SegWitV0 | ConvertInfo::NestedV0 => {
                let ms = Miniscript::<_, miniscript::Segwitv0>::from_ast(
                    miniscript::Terminal::Multi(
                        self.threshold(),
                        self.pubkeys.clone(),
                    ),
                )
                .expect("miniscript is unable to produce mutisig");
                let ms = ms.translate_pk2(|pk| {
                    if pk.is_uncompressed() {
                        return Err(Error::UncompressedKeyInSegWitContext);
                    }
                    Ok(pk.derive_public_key(ctx, child_index))
                })?;
                Ok(ms.encode().into())
            }
            ConvertInfo::Taproot { .. } => unimplemented!(),
            _ => {
                let ms = Miniscript::<_, miniscript::Legacy>::from_ast(
                    miniscript::Terminal::Multi(
                        self.threshold(),
                        self.pubkeys.clone(),
                    ),
                )
                .expect("miniscript is unable to produce mutisig");
                let ms = ms.translate_pk2_infallible(|pk| {
                    pk.derive_public_key(ctx, child_index)
                });
                Ok(ms.encode().into())
            }
        }
    }
}

impl DeriveLockScript for MuSigBranched {
    fn derive_lock_script<C: Verification>(
        &self,
        _ctx: &Secp256k1<C>,
        _child_index: UnhardenedIndex,
        _descr_category: ConvertInfo,
    ) -> Result<LockScript, Error> {
        // TODO: Implement after Taproot release
        unimplemented!()
    }
}

impl DeriveLockScript for Template {
    fn derive_lock_script<C: Verification>(
        &self,
        ctx: &Secp256k1<C>,
        child_index: UnhardenedIndex,
        descr_category: ConvertInfo,
    ) -> Result<LockScript, Error> {
        match self {
            Template::SingleSig(_) => Err(Error::SingleSig),
            Template::MultiSig(multisig) => {
                multisig.derive_lock_script(ctx, child_index, descr_category)
            }
            Template::Scripted(scripted) => {
                scripted.derive_lock_script(ctx, child_index, descr_category)
            }
            Template::MuSigBranched(musig) => {
                musig.derive_lock_script(ctx, child_index, descr_category)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use bitcoin::util::bip32;
    use miniscript::descriptor::DescriptorSinglePub;

    use super::SingleSigDescriptorParts;
    use crate::SingleSig;

    #[test]
    fn singlesigdescriptorparts_from_str_returns_pubkey() {
        let descriptor = "pk(0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798))";

        let expected = Some(SingleSigDescriptorParts {
            pubkey: "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
            fingerprint: None,
            derivation: None,
        });
        let result = SingleSigDescriptorParts::from_str(descriptor);

        assert_eq!(result, expected);
    }

    #[test]
    fn singlesigdescriptorparts_from_str_returns_pubkey_and_fingerprint() {
        let descriptor = "pkh([d34db33f/44'/0'/0'\
                          ]02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5)";

        let expected = Some(SingleSigDescriptorParts {
            pubkey: "02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5",
            fingerprint: Some("d34db33f"),
            derivation: Some("44'/0'/0'"),
        });
        let result = SingleSigDescriptorParts::from_str(descriptor);

        assert_eq!(result, expected);
    }

    #[test]
    fn singlesigdescriptorparts_from_str_returns_pubkey_when_fingerprint_invalid(
    ) {
        let descriptor = "pkh([qwer/44'/0'/0'\
                          ]02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5)";

        let expected = Some(SingleSigDescriptorParts {
            pubkey: "02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5",
            fingerprint: None,
            derivation: None,
        });
        let result = SingleSigDescriptorParts::from_str(descriptor);

        assert_eq!(result, expected);
    }

    #[test]
    fn singlesigdescriptorparts_from_str_returns_pubkey_when_only_pubkey() {
        let descriptor = "02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5";

        let expected = Some(SingleSigDescriptorParts {
            pubkey: "02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5",
            fingerprint: None,
            derivation: None,
        });
        let result = SingleSigDescriptorParts::from_str(descriptor);

        assert_eq!(result, expected);
    }

    #[test]
    fn singlesigdescriptorparts_from_str_returns_none_when_pubkey_invalid() {
        let descriptor = "pk(0279be667ef9dcbbac55a06295ce870b07INVALID))";

        let expected = None::<SingleSigDescriptorParts>;
        let result = SingleSigDescriptorParts::from_str(descriptor);

        assert_eq!(result, expected);
    }

    #[test]
    fn singlesig_from_str_returns_pubkey() {
        let descriptor = "pk(0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798))";

        let origin = None::<(bip32::Fingerprint, bip32::DerivationPath)>;
        let key = bitcoin::PublicKey::from_str(
            "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
        )
            .unwrap();

        let expected = SingleSig::Pubkey(DescriptorSinglePub { origin, key });
        let result = SingleSig::from_str(descriptor).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn singlesig_from_str_returns_pubkey_and_fingerprint() {
        let descriptor = "pkh([d34db33f/44'/0'/0'\
                          ]02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5)";

        let fingerprint = "d34db33f".parse::<bip32::Fingerprint>().unwrap();
        let path = bip32::DerivationPath::from_str("m/44'/0'/0'").unwrap();
        let origin = Some((fingerprint, path));
        let key = bitcoin::PublicKey::from_str(
            "02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5",
        )
            .unwrap();

        let expected = SingleSig::Pubkey(DescriptorSinglePub { origin, key });
        let result = SingleSig::from_str(descriptor).unwrap();

        assert_eq!(result, expected);
    }
}
