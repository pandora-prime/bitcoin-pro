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

use bitcoin::Script;
use wallet::bip32::UnhardenedIndex;
use wallet::descriptor;

#[derive(Clone, PartialEq, Eq, Debug, StrictEncode, StrictDecode)]
#[strict_encoding_crate(lnpbp::strict_encoding)]
pub struct DescriptorAccount {
    pub name: String,
    pub generator: descriptor::Generator,
}

impl DescriptorAccount {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn type_name(&self) -> String {
        match self.generator.template {
            descriptor::Template::SingleSig(_) => s!("Single-sig."),
            descriptor::Template::MultiSig(_) => s!("Multi-sig."),
            descriptor::Template::Scripted(_) => s!("Custom script"),
            descriptor::Template::MuSigBranched(_) => s!("Tapscript"),
            _ => s!("Unsupported"),
        }
    }

    pub fn descriptor(&self) -> String {
        self.generator.to_string()
    }

    pub fn pubkey_scripts_count(&self) -> u32 {
        self.generator.variants.count()
    }

    pub fn pubkey_scripts(
        &self,
        index: UnhardenedIndex,
    ) -> Result<HashMap<descriptor::Category, Script>, descriptor::Error> {
        self.generator.pubkey_scripts(index)
    }
}
