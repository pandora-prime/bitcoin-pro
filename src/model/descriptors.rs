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

use std::collections::HashMap;
use std::str::FromStr;

use lnpbp::bitcoin::{self, blockdata::script::Error as ScriptError, Script};
use lnpbp::bp::DescriptorCategory;
use lnpbp::hex::{self, FromHex};
use lnpbp::miniscript::{
    self, Descriptor, Miniscript, NullCtx, ScriptContext, Terminal,
};

use super::TrackingKey;

// TODO: Consider moving to LNP/BP Core Library

#[derive(Clone, PartialEq, Eq, Debug, Display, From, Error)]
#[display(doc_comments)]
pub enum Error {
    /// Hex encoding error: {0}
    #[from]
    Hex(hex::Error),

    /// Bitcoin script error: {0}
    #[from]
    Script(ScriptError),

    /// Miniscript error
    #[display("{0}")]
    Miniscript(String),
}

impl From<miniscript::Error> for Error {
    fn from(err: miniscript::Error) -> Self {
        Error::Miniscript(err.to_string())
    }
}

#[derive(Clone, PartialEq, Eq, Debug, StrictEncode, StrictDecode)]
pub struct DescriptorGenerator {
    pub name: String,
    pub content: DescriptorContent,
    pub types: DescriptorTypes,
}

impl DescriptorGenerator {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn type_name(&self) -> String {
        match self.content {
            DescriptorContent::SingleSig(_) => s!("Single-sig."),
            DescriptorContent::MultiSig(_, _) => s!("Multi-sig."),
            DescriptorContent::LockScript(_, _) => s!("Custom script"),
        }
    }

    pub fn descriptor(&self) -> String {
        let single = self.content.is_singlesig();
        let mut d = vec![];
        if self.types.bare {
            d.push(if single { "pk" } else { "bare" });
        }
        if self.types.hashed {
            d.push(if single { "pkh" } else { "sh" });
        }
        if self.types.nested {
            d.push(if single { "sh_wpkh" } else { "sh_wsh" });
        }
        if self.types.segwit {
            d.push(if single { "wpkh" } else { "wsh" });
        }
        if self.types.taproot {
            d.push("tpk");
        }
        let data = match &self.content {
            DescriptorContent::SingleSig(key) => key.to_string(),
            DescriptorContent::MultiSig(threshold, keyset) => {
                format!(
                    "thresh_m({},{})",
                    threshold,
                    keyset
                        .iter()
                        .map(TrackingKey::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
            DescriptorContent::LockScript(_, script) => script.clone(),
        };
        format!("{}({})", d.join("|"), data)
    }

    pub fn pubkey_scripts_count(&self) -> u32 {
        self.types.bare as u32
            + self.types.hashed as u32
            + self.types.nested as u32
            + self.types.segwit as u32
            + self.types.taproot as u32
    }

    pub fn pubkey_scripts(
        &self,
        index: u32,
    ) -> Result<HashMap<DescriptorCategory, Script>, Error> {
        let mut scripts = HashMap::with_capacity(5);
        let single = if let DescriptorContent::SingleSig(_) = self.content {
            Some(self.content.public_key(index).expect("Can't fail"))
        } else {
            None
        };
        if self.types.bare {
            let d = if let Some(pk) = single {
                Descriptor::Pk(pk)
            } else {
                Descriptor::Bare(self.content.miniscript(index)?)
            };
            scripts.insert(DescriptorCategory::Bare, d.script_pubkey(NullCtx));
        }
        if self.types.hashed {
            let d = if let Some(pk) = single {
                Descriptor::Pkh(pk)
            } else {
                Descriptor::Sh(self.content.miniscript(index)?)
            };
            scripts
                .insert(DescriptorCategory::Hashed, d.script_pubkey(NullCtx));
        }
        if self.types.nested {
            let d = if let Some(pk) = single {
                Descriptor::ShWpkh(pk)
            } else {
                Descriptor::ShWsh(self.content.miniscript(index)?)
            };
            scripts
                .insert(DescriptorCategory::Nested, d.script_pubkey(NullCtx));
        }
        if self.types.segwit {
            let d = if let Some(pk) = single {
                Descriptor::Wpkh(pk)
            } else {
                Descriptor::Wsh(self.content.miniscript(index)?)
            };
            scripts
                .insert(DescriptorCategory::SegWit, d.script_pubkey(NullCtx));
        }
        /* TODO: Enable once Taproot will go live
        if self.taproot {
            scripts.push(content.taproot());
        }
         */
        Ok(scripts)
    }
}

#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Debug, StrictEncode, StrictDecode,
)]
pub struct DescriptorTypes {
    pub bare: bool,
    pub hashed: bool,
    pub nested: bool,
    pub segwit: bool,
    pub taproot: bool,
}

impl DescriptorTypes {
    pub fn has_match(&self, descriptor_type: DescriptorCategory) -> bool {
        match descriptor_type {
            DescriptorCategory::Bare => self.bare,
            DescriptorCategory::Hashed => self.hashed,
            DescriptorCategory::Nested => self.nested,
            DescriptorCategory::SegWit => self.segwit,
            DescriptorCategory::Taproot => self.taproot,
            _ => false,
        }
    }
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Hash,
    StrictEncode,
    StrictDecode,
)]
pub enum DescriptorContent {
    SingleSig(TrackingKey),
    MultiSig(u8, Vec<TrackingKey>),
    LockScript(SourceType, String),
}

impl DescriptorContent {
    pub fn is_singlesig(&self) -> bool {
        match self {
            DescriptorContent::SingleSig(_) => true,
            _ => false,
        }
    }

    pub fn public_key(&self, index: u32) -> Option<bitcoin::PublicKey> {
        match self {
            DescriptorContent::SingleSig(key) => Some(key.public_key(index)),
            _ => None,
        }
    }

    pub fn miniscript<Ctx>(
        &self,
        index: u32,
    ) -> Result<Miniscript<bitcoin::PublicKey, Ctx>, Error>
    where
        Ctx: ScriptContext,
    {
        Ok(match self {
            DescriptorContent::SingleSig(key) => {
                let pk = key.public_key(index);
                Miniscript::from_ast(Terminal::PkK(pk))?
            }
            DescriptorContent::MultiSig(thresh, keyset) => {
                let ks = keyset
                    .into_iter()
                    .map(|key| key.public_key(index))
                    .collect();
                Miniscript::from_ast(Terminal::Multi(*thresh as usize, ks))?
            }
            DescriptorContent::LockScript(source_type, script) => {
                match source_type {
                    SourceType::Binary => {
                        let script = Script::from(Vec::from_hex(script)?);
                        Miniscript::parse(&script)?
                    }
                    SourceType::Assembly => {
                        // TODO: Parse assembly
                        let script = Script::from(Vec::from_hex(script)?);
                        Miniscript::parse(&script)?
                    }
                    SourceType::Miniscript => Miniscript::from_str(script)?,
                    SourceType::Policy => {
                        // TODO: Compiler will require changes to LNP/BP
                        // policy::Concrete::from_str(script)?.compile()?
                        Miniscript::from_str(script)?
                    }
                }
            }
        })
    }
}

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
pub enum SourceType {
    Binary,
    Assembly,
    Miniscript,
    Policy,
}
