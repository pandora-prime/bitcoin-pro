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
