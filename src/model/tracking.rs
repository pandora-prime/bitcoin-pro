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

use wallet::descriptor;

#[derive(Getters, Clone, PartialEq, Eq, Debug, StrictEncode, StrictDecode)]
#[strict_encoding_crate(lnpbp::strict_encoding)]
pub struct TrackingAccount {
    pub name: String,
    pub key: descriptor::SingleSig,
}

impl TrackingAccount {
    pub fn details(&self) -> String {
        self.key.to_string()
    }

    pub fn count(&self) -> u32 {
        self.key.count()
    }
}
