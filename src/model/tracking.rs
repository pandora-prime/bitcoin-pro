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

use std::io;
use std::ops::RangeInclusive;

use lnpbp::bitcoin::util::bip32::{ExtendedPubKey, KeySource};
use lnpbp::secp256k1;
use lnpbp::strict_encoding::{self, StrictDecode, StrictEncode};

#[derive(Clone, PartialEq, Eq, Debug, StrictEncode, StrictDecode)]
pub struct TrackingAccount {
    pub name: String,
    pub key: TrackingKey,
}

#[derive(Clone, PartialEq, Eq, Debug, StrictEncode, StrictDecode)]
pub enum TrackingKey {
    SingleKey(secp256k1::PublicKey),
    HdKeySet(DerivationComponents),
}

// TODO: Consider moving to LNP/BP Core library
#[derive(Clone, PartialEq, Eq, Debug, StrictEncode, StrictDecode)]
pub struct DerivationComponents {
    pub branch_xpub: ExtendedPubKey,
    pub branch_source: KeySource,
    pub terminal_path: Vec<u32>,
    pub index_ranges: Vec<DerivationRange>,
}

#[derive(Wrapper, Clone, PartialEq, Eq, Debug, From)]
pub struct DerivationRange(RangeInclusive<u32>);

impl StrictEncode for DerivationRange {
    type Error = strict_encoding::Error;

    fn strict_encode<E: io::Write>(&self, e: E) -> Result<usize, Self::Error> {
        unimplemented!()
    }
}

impl StrictDecode for DerivationRange {
    type Error = strict_encoding::Error;

    fn strict_decode<D: io::Read>(d: D) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}
