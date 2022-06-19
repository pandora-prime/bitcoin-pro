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

use bitcoin::OutPoint;
use electrum_client::ListUnspentRes;
use wallet::descriptor;

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
#[strict_encoding_crate(lnpbp::strict_encoding)]
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
