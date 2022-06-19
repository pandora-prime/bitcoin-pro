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
use wallet::descriptors;
use wallet::hd::UnhardenedIndex;

#[derive(Clone, PartialEq, Eq, Debug, StrictEncode, StrictDecode)]
pub struct DescriptorAccount {
    pub name: String,
    pub generator: descriptors::Generator,
}

impl DescriptorAccount {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn type_name(&self) -> String {
        match self.generator.template {
            descriptors::Template::SingleSig(_) => s!("Single-sig."),
            descriptors::Template::MultiSig(_) => s!("Multi-sig."),
            descriptors::Template::Scripted(_) => s!("Custom script"),
            descriptors::Template::MuSigBranched(_) => s!("Tapscript"),
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
    ) -> Result<HashMap<descriptors::Category, Script>, descriptors::Error>
    {
        self.generator.pubkey_scripts(index)
    }
}
