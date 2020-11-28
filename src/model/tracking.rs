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

use std::fmt::{self, Display, Formatter};
use std::io;
use std::iter::FromIterator;
use std::ops::RangeInclusive;

use amplify::Wrapper;
use lnpbp::bitcoin::util::bip32::{
    ChildNumber, DerivationPath, ExtendedPubKey,
};
use lnpbp::secp256k1;
use lnpbp::strict_encoding::{self, StrictDecode, StrictEncode};

#[derive(Getters, Clone, PartialEq, Eq, Debug, StrictEncode, StrictDecode)]
pub struct TrackingAccount {
    pub name: String,
    pub key: TrackingKey,
}

impl TrackingAccount {
    pub fn details(&self) -> String {
        self.key.details()
    }

    pub fn count(&self) -> u32 {
        self.key.count()
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Display, StrictEncode, StrictDecode)]
#[display(TrackingKey::details)]
pub enum TrackingKey {
    SingleKey(secp256k1::PublicKey),
    HdKeySet(DerivationComponents),
}

impl TrackingKey {
    pub fn details(&self) -> String {
        match self {
            TrackingKey::SingleKey(ref pubkey) => pubkey.to_string(),
            TrackingKey::HdKeySet(ref keyset) => keyset.to_string(),
        }
    }

    pub fn count(&self) -> u32 {
        match self {
            TrackingKey::SingleKey(_) => 1,
            TrackingKey::HdKeySet(ref keyset) => keyset.count(),
        }
    }
}

// TODO: Consider moving the rest of the file to LNP/BP Core library
pub trait HardenedNormalSplit {
    fn hardened_normal_split(&self) -> (DerivationPath, Vec<u32>);
}

impl HardenedNormalSplit for DerivationPath {
    fn hardened_normal_split(&self) -> (DerivationPath, Vec<u32>) {
        let mut terminal_path = vec![];
        let branch_path = self
            .into_iter()
            .rev()
            .by_ref()
            .skip_while(|child| {
                if let ChildNumber::Normal { index } = child {
                    terminal_path.push(index);
                    true
                } else {
                    false
                }
            })
            .cloned()
            .collect::<DerivationPath>();
        let branch_path = branch_path.into_iter().rev().cloned().collect();
        let terminal_path = terminal_path.into_iter().rev().cloned().collect();
        (branch_path, terminal_path)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, StrictEncode, StrictDecode)]
// master_xpub/branch_path=branch_xpub/terminal_path/index_ranges
pub struct DerivationComponents {
    pub master_xpub: ExtendedPubKey,
    pub branch_path: DerivationPath,
    pub branch_xpub: ExtendedPubKey,
    pub terminal_path: Vec<u32>,
    pub index_ranges: Option<Vec<DerivationRange>>,
}

impl DerivationComponents {
    pub fn count(&self) -> u32 {
        match self.index_ranges {
            None => u32::MAX,
            Some(ref ranges) => {
                ranges.iter().fold(0u32, |sum, range| sum + range.count())
            }
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
            .map(|ranges| {
                ranges
                    .iter()
                    .map(DerivationRange::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_default()
    }

    pub fn child(&self, child: u32) -> ExtendedPubKey {
        let derivation = self
            .terminal_path()
            .into_child(ChildNumber::Normal { index: child });
        self.branch_xpub
            .derive_pub(&lnpbp::SECP256K1, &derivation)
            .expect("Non-hardened derivation does not fail")
    }
}

impl Display for DerivationComponents {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}]{}/",
            self.master_xpub.fingerprint(),
            self.derivation_path()
                .to_string()
                .strip_prefix("m")
                .unwrap_or(&self.derivation_path().to_string())
        )?;
        if let Some(ref ranges) = self.index_ranges {
            f.write_str(&self.index_ranges_string())
        } else {
            f.write_str("*")
        }
    }
}

#[derive(Wrapper, Clone, PartialEq, Eq, Debug, From)]
pub struct DerivationRange(RangeInclusive<u32>);

impl DerivationRange {
    pub fn count(&self) -> u32 {
        let inner = self.as_inner();
        inner.end() - inner.start() + 1
    }

    pub fn start(&self) -> u32 {
        *self.as_inner().start()
    }

    pub fn end(&self) -> u32 {
        *self.as_inner().end()
    }
}

impl Display for DerivationRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let inner = self.as_inner();
        if inner.start() == inner.end() {
            write!(f, "{}", inner.start())
        } else {
            write!(f, "{}-{}", inner.start(), inner.end())
        }
    }
}

impl StrictEncode for DerivationRange {
    type Error = strict_encoding::Error;

    fn strict_encode<E: io::Write>(
        &self,
        mut e: E,
    ) -> Result<usize, Self::Error> {
        Ok(strict_encode_list!(e; self.start(), self.end()))
    }
}

impl StrictDecode for DerivationRange {
    type Error = strict_encoding::Error;

    fn strict_decode<D: io::Read>(mut d: D) -> Result<Self, Self::Error> {
        Ok(Self::from_inner(RangeInclusive::new(
            u32::strict_decode(&mut d)?,
            u32::strict_decode(&mut d)?,
        )))
    }
}
