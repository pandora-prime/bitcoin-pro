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
use lnpbp::bp::descriptor;

use super::DescriptorAccount;

#[derive(
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[display("{amount}@{outpoint} {descriptor_category}({descriptor_template})")]
pub struct UtxoEntry {
    pub outpoint: OutPoint,
    pub height: u32,
    pub amount: u64,
    pub descriptor_template: descriptor::Template,
    pub descriptor_category: descriptor::Category,
    pub derivation_index: u32,
}

impl UtxoEntry {
    pub fn with(
        res: &ListUnspentRes,
        descriptor_template: descriptor::Template,
        descriptor_category: descriptor::Category,
        derivation_index: u32,
    ) -> Self {
        UtxoEntry {
            outpoint: OutPoint {
                txid: res.tx_hash,
                vout: res.tx_pos as u32,
            },
            height: res.height as u32,
            amount: res.value,
            descriptor_template,
            descriptor_category,
            derivation_index,
        }
    }

    pub fn has_match(&self, descriptor_account: &DescriptorAccount) -> bool {
        descriptor_account.generator.template == self.descriptor_template
            && descriptor_account
                .generator
                .variants
                .has_match(self.descriptor_category)
    }
}
