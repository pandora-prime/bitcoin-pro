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

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename = "lowercase")
)]
#[derive(
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[non_exhaustive]
#[display(inner)]
#[allow(clippy::large_enum_variant)]
pub enum ScriptConstruction {
    #[cfg_attr(feature = "serde", serde(rename = "script"))]
    ScriptTemplate(ScriptTemplate<SingleSig>),

    Miniscript(
        #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
        Miniscript<SingleSig, miniscript::Segwitv0>,
    ),

    #[cfg_attr(feature = "serde", serde(rename = "policy"))]
    MiniscriptPolicy(
        #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
        policy::Concrete<SingleSig>,
    ),
}

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[derive(
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Debug,
    StrictEncode,
    StrictDecode,
)]
pub struct ScriptSource {
    pub script: ScriptConstruction,

    pub source: Option<String>,

    #[cfg_attr(
        feature = "serde",
        serde(with = "As::<Option<DisplayFromStr>>")
    )]
    pub tweak_target: Option<SingleSig>,
}

impl Display for ScriptSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(ref source) = self.source {
            f.write_str(source)
        } else {
            Display::fmt(&self.script, f)
        }
    }
}

/// Representation formats for bitcoin script data
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[cfg_attr(feature = "clap", Clap)]
#[cfg_attr(feature = "strict_encoding", derive(StrictEncode, StrictDecode))]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename = "lowercase")
)]
#[non_exhaustive]
pub enum ScriptSourceFormat {
    /// Binary script source encoded as hexadecimal string
    #[display("hex")]
    Hex,

    /// Binary script source encoded as Base64 string
    #[display("base64")]
    Base64,

    /// Miniscript string or descriptor
    #[display("miniscript")]
    Miniscript,

    /// Miniscript string or descriptor
    #[display("policy")]
    Policy,

    /// String with assembler opcodes
    #[display("asm")]
    Asm,
}
