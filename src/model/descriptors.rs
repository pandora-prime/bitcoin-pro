// Bitcoin Pro: Professional bitcoin accounts & assets management
// Written in 2020-2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

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
