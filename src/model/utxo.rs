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

use electrum_client::ListUnspentRes;
use lnpbp::bitcoin::OutPoint;

#[derive(
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Display,
    Serialize,
    Deserialize,
    StrictEncode,
    StrictDecode,
)]
#[display("{amount}@{outpoint}")]
pub struct UtxoEntry {
    pub outpoint: OutPoint,
    pub height: u32,
    pub amount: u64,
}

impl From<&ListUnspentRes> for UtxoEntry {
    fn from(res: &ListUnspentRes) -> Self {
        UtxoEntry {
            outpoint: OutPoint {
                txid: res.tx_hash,
                vout: res.tx_pos as u32,
            },
            height: res.height as u32,
            amount: res.value,
        }
    }
}

impl From<ListUnspentRes> for UtxoEntry {
    fn from(res: ListUnspentRes) -> Self {
        UtxoEntry::from(&res)
    }
}
