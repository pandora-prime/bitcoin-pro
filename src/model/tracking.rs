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

use std::collections::HashSet;
use std::ops::RangeInclusive;

use lnpbp::bitcoin::util::bip32::{DerivationPath, ExtendedPubKey, KeySource};
use lnpbp::secp256k1;

#[derive(
    Clone,
    PartialEq,
    Eq,
    Debug,
    Display,
    Serialize,
    Deserialize,
    StrictEncoding,
    StrictDecoding,
)]
#[display("tracking account '{name}' {key}")]
pub struct TrackingAccount {
    pub name: String,
    pub key: TrackingKey,
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    Debug,
    Display,
    Serialize,
    Deserialize,
    StrictEncoding,
    StrictDecoding,
)]
pub enum TrackingKey {
    #[display("pubkey {0}")]
    SingleKey(secp256k1::PublicKey),
    #[display("keyset {0}")]
    HdKeySet(DerivationComponents),
}

// TODO: Consider moving to LNP/BP Core library
#[derive(
    Clone,
    PartialEq,
    Eq,
    Debug,
    Display,
    Serialize,
    Deserialize,
    StrictEncoding,
    StrictDecoding,
)]
#[display("{branch_source}={branch_xpub}")]
pub struct DerivationComponents {
    pub branch_xpub: ExtendedPubKey,
    pub branch_source: KeySource,
    pub terminal_path: Vec<u32>,
    pub index_ranges: HashSet<RangeInclusive<u32>>,
}
