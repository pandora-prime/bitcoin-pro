// Descriptor wallet library extending bitcoin & miniscript functionality
// by LNP/BP Association (https://lnp-bp.org)
// Written in 2020-2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the Apache-2.0 License
// along with this software.
// If not, see <https://opensource.org/licenses/Apache-2.0>.

use std::fmt::{self, Display, Formatter};
use std::iter::FromIterator;
use std::str::FromStr;

use bitcoin::secp256k1::{self, Secp256k1, Verification};
use bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPubKey};
use miniscript::MiniscriptKey;
use slip132::FromSlip132;
use strict_encoding::{self, StrictDecode, StrictEncode};

use super::{DerivationRangeVec, HardenedNormalSplit, UnhardenedIndex};

#[derive(
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    StrictEncode,
    StrictDecode,
)]
// [master_xpub]/branch_path=[branch_xpub]/terminal_path/index_ranges
pub struct DerivationComponents {
    pub master_xpub: ExtendedPubKey,
    pub branch_path: DerivationPath,
    pub branch_xpub: ExtendedPubKey,
    pub terminal_path: Vec<u32>,
    pub index_ranges: Option<DerivationRangeVec>,
}

impl DerivationComponents {
    pub fn count(&self) -> u32 {
        match self.index_ranges {
            None => ::std::u32::MAX,
            Some(ref ranges) => ranges.count(),
        }
    }

    pub fn derivation_path(&self) -> DerivationPath {
        self.branch_path.extend(self.terminal_path())
    }

    pub fn terminal_path(&self) -> DerivationPath {
        DerivationPath::from_iter(
            self.terminal_path
                .iter()
                .map(|i| ChildNumber::Normal { index: *i }),
        )
    }

    pub fn index_ranges_string(&self) -> String {
        self.index_ranges
            .as_ref()
            .map(DerivationRangeVec::to_string)
            .unwrap_or_default()
    }

    pub fn child<C: Verification>(
        &self,
        ctx: &Secp256k1<C>,
        child: u32,
    ) -> ExtendedPubKey {
        let derivation = self
            .terminal_path()
            .into_child(ChildNumber::Normal { index: child });
        self.branch_xpub
            .derive_pub(ctx, &derivation)
            .expect("Non-hardened derivation does not fail")
    }

    pub fn derive_public_key<C: Verification>(
        &self,
        ctx: &Secp256k1<C>,
        child_index: UnhardenedIndex,
    ) -> secp256k1::PublicKey {
        self.child(ctx, child_index.into()).public_key
    }
}

impl Display for DerivationComponents {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "[{}]", self.master_xpub.fingerprint())?;
        } else {
            write!(f, "[{}]", self.master_xpub)?;
        }
        f.write_str(self.branch_path.to_string().trim_start_matches('m'))?;
        if f.alternate() {
            f.write_str("/")?;
        } else if self.branch_xpub != self.master_xpub {
            write!(f, "=[{}]", self.branch_xpub)?;
        }
        f.write_str(self.terminal_path().to_string().trim_start_matches('m'))?;
        f.write_str("/")?;
        if self.index_ranges.is_some() {
            f.write_str(&self.index_ranges_string())
        } else {
            f.write_str("*")
        }
    }
}

// TODO: #22 Re-org error into an enum
#[derive(
    Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, Error,
)]
#[display(inner)]
pub struct ComponentsParseError(pub String);

impl FromStr for DerivationComponents {
    type Err = ComponentsParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split('=');
        let (branch, terminal) = match (split.next(), split.next(), split.next()) {
            (Some(branch), Some(terminal), None) => (Some(branch), terminal),
            (Some(terminal), None, None) => (None, terminal),
            (None, None, None) => unreachable!(),
            _ => {
                return Err(ComponentsParseError(s!("Derivation components string \
                                                    must contain at most two parts \
                                                    separated by `=`")))
            }
        };

        let caps = if let Some(caps) = DerivationStringParts::from_str(terminal)
        {
            caps
        } else {
            return Err(ComponentsParseError(s!(
                "Wrong composition of derivation components data"
            )));
        };

        let branch_xpub = ExtendedPubKey::from_slip132_str(caps.xpub)
            .map_err(|err| ComponentsParseError(err.to_string()))?;
        let terminal_path = caps.derivation;
        let terminal_path =
            DerivationPath::from_str(&format!("m/{}", terminal_path))
                .map_err(|err| ComponentsParseError(err.to_string()))?;
        let (prefix, terminal_path) = terminal_path.hardened_normal_split();
        if !prefix.as_ref().is_empty() {
            return Err(ComponentsParseError(s!(
                "Terminal derivation path must not contain hardened keys"
            )));
        }
        let index_ranges = caps
            .range
            .map(DerivationRangeVec::from_str)
            .transpose()
            .map_err(|err| ComponentsParseError(err.to_string()))?;

        let (master_xpub, branch_path) = if let Some(caps) =
            branch.and_then(DerivationStringParts::from_str)
        {
            let master_xpub = ExtendedPubKey::from_slip132_str(caps.xpub)
                .map_err(|err| ComponentsParseError(err.to_string()))?;
            let branch_path = caps.derivation;
            let branch_path =
                DerivationPath::from_str(&format!("m/{}", branch_path))
                    .map_err(|err| ComponentsParseError(err.to_string()))?;
            (master_xpub, branch_path)
        } else {
            (branch_xpub, DerivationPath::from(Vec::<ChildNumber>::new()))
        };

        Ok(DerivationComponents {
            master_xpub,
            branch_path,
            branch_xpub,
            terminal_path,
            index_ranges,
        })
    }
}

impl MiniscriptKey for DerivationComponents {
    type Hash = Self;

    fn to_pubkeyhash(&self) -> Self::Hash {
        self.clone()
    }
}

/// Components of a [DerivationComponents] string.
/// Only used to split it into its different parts.
#[derive(Debug, PartialEq, Eq)]
struct DerivationStringParts<'a> {
    /// xpub
    xpub: &'a str,
    /// Derivation path
    derivation: &'a str,
    /// Derivation range
    range: Option<&'a str>,
}

impl<'a> DerivationStringParts<'a> {
    /// Attempts to split `s` into its [DerivationStringParts].
    /// `None` will be returned if `s` doesn't match the pattern.
    fn from_str(s: &'a str) -> Option<DerivationStringParts> {
        let mut closing = None::<usize>;
        for (i, c) in s.chars().enumerate() {
            // Check xpub until closing bracket is found
            if closing.is_none() {
                match i {
                    0 => {
                        if c != '[' {
                            return None;
                        }
                    }
                    // xpub content
                    1..=111 => {
                        if "0IOl".contains(c) || !c.is_ascii_alphanumeric() {
                            return None;
                        }
                    }
                    // Closing bracket
                    112 | 113 => {
                        if c == ']' {
                            closing = Some(i);
                        }
                    }
                    // xpub is too long
                    _ => return None,
                }
                continue;
            }

            // Check derivation path and optional range
            if let Some(closing) = closing {
                let from_closing = i - closing;
                match from_closing {
                    1 => {
                        if c != '/' {
                            return None;
                        }
                    }
                    2 => {
                        if c != '*' && !c.is_ascii_digit() {
                            return None;
                        }
                    }
                    // Check invalid characters
                    _ => {
                        if !"*/h',-".contains(c) && !c.is_ascii_digit() {
                            return None;
                        }
                    }
                }
            }
        }

        // Determine path and range
        if let Some(closing) = closing {
            // Can't end on xpub's closing bracket
            if s.len() <= closing + 1 {
                return None;
            }

            if let Some(maybe_range) = s[closing..].rfind(|c| "*,-".contains(c))
            {
                let maybe_range = maybe_range + closing;
                // Check if '*' range
                if s[maybe_range..].starts_with('*')
                    && s[maybe_range..].len() != 1
                {
                    // A range can only have one '*'
                    return None;
                } else if &s[maybe_range..] == "*" {
                    return Some(Self {
                        xpub: &s[1..closing],
                        derivation: &s[closing + 2..maybe_range - 1],
                        range: Some(&s[maybe_range..]),
                    });
                }

                // It's a x-y or x,y range
                // closing + 2 to skip "]/"
                if let Some(real_range) = s[closing + 2..].rfind('/') {
                    let real_range = real_range + closing + 2;
                    return Some(Self {
                        xpub: &s[1..closing],
                        derivation: &s[closing + 2..real_range],
                        range: Some(&s[real_range + 1..]),
                    });
                } else {
                    // "*,-" needs to occur after a second '/'
                    return None;
                }
            } else {
                // No range detected
                return Some(Self {
                    xpub: &s[1..closing],
                    derivation: &s[closing + 2..],
                    range: None,
                });
            }
        }

        // Bracket enclosed xpub is not present
        None
    }
}

#[cfg(test)]
mod test {
    use super::DerivationStringParts;

    #[test]
    fn derivationstringparts_from_str_returns_xpub_and_derivation() {
        let derivation = "[xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw]/1";
        let hardened_derivation = "[xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw]/1/878'/1971h/420";
        let no_derivation = "[xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw]";

        let expected_derivation = Some(DerivationStringParts{
            xpub: "xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw",
            derivation: "1",
            range: None,
        });
        let expected_hardened_derivation = Some(DerivationStringParts{
            xpub: "xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw",
            derivation: "1/878'/1971h/420",
            range: None,
        });
        let expected_no_derivation = None::<DerivationStringParts>;

        let result_derivation = DerivationStringParts::from_str(derivation);
        let result_hardened_derivation =
            DerivationStringParts::from_str(hardened_derivation);
        let result_no_derivation =
            DerivationStringParts::from_str(no_derivation);

        assert_eq!(result_derivation, expected_derivation);
        assert_eq!(result_hardened_derivation, expected_hardened_derivation);
        assert_eq!(result_no_derivation, expected_no_derivation);
    }

    #[test]
    fn derivationstringparts_from_str_returns_xpub_derivation_and_range() {
        let hyphen_range = "[xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw]/1/878'/1971h/420/0-1000";
        let comma_range = "[xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw]/1/878'/1971h/420/0,1000";
        let star_range = "[xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw]/1/878'/1971h/420/*";
        let invalid_range = "[xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw]/1/878'/1971h/420/0#1000";

        let expected_hyphen = Some(DerivationStringParts{
            xpub: "xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw",
            derivation: "1/878'/1971h/420",
            range: Some("0-1000"),
        });
        let expected_comma = Some(DerivationStringParts{
            xpub: "xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw",
            derivation: "1/878'/1971h/420",
            range: Some("0,1000"),
        });
        let expected_star = Some(DerivationStringParts{
            xpub: "xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw",
            derivation: "1/878'/1971h/420",
            range: Some("*"),
        });
        let expected_invalid = None::<DerivationStringParts>;

        let result_hyphen = DerivationStringParts::from_str(hyphen_range);
        let result_comma = DerivationStringParts::from_str(comma_range);
        let result_star = DerivationStringParts::from_str(star_range);
        let result_invalid = DerivationStringParts::from_str(invalid_range);

        assert_eq!(result_hyphen, expected_hyphen);
        assert_eq!(result_comma, expected_comma);
        assert_eq!(result_star, expected_star);
        assert_eq!(result_invalid, expected_invalid);
    }

    #[test]
    fn derivationstringparts_from_str_returns_none_when_empty() {
        let s = "";

        let expected = None::<DerivationStringParts>;
        let result = DerivationStringParts::from_str(s);

        assert_eq!(result, expected);
    }

    #[test]
    fn derivationstringparts_from_str_returns_none_when_bracket_missing() {
        let no_brackets = "xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw/1/878'/1971h/420/10,1000";
        let no_left_bracket = "xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw]/1/878'/1971h/420/10,1000";
        let no_right_bracket = "[xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw/1/878'/1971h/420/10,1000";

        let expected = None::<DerivationStringParts>;
        let result_no_brackets = DerivationStringParts::from_str(no_brackets);
        let result_left_bracket =
            DerivationStringParts::from_str(no_left_bracket);
        let result_right_bracket =
            DerivationStringParts::from_str(no_right_bracket);

        assert_eq!(result_no_brackets, expected);
        assert_eq!(result_left_bracket, expected);
        assert_eq!(result_right_bracket, expected);
    }

    #[test]
    fn derivationstringparts_from_str_returns_none_when_wrong_xpub_len() {
        let too_short = "xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDn/1/878'/1971h/420/10,1000";
        let too_long = "xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnwww]/1/878'/1971h/420/10,1000";

        let expected = None::<DerivationStringParts>;
        let result_too_short = DerivationStringParts::from_str(too_short);
        let result_too_long = DerivationStringParts::from_str(too_long);

        assert_eq!(result_too_short, expected);
        assert_eq!(result_too_long, expected);
    }
}
