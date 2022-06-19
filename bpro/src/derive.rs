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

use bitcoin::secp256k1::{Secp256k1, Verification};
use bitcoin::Script;
use bitcoin_hd::{DerivePublicKey, UnhardenedIndex};
use bitcoin_scripts::{ConvertInfo, LockScript};
use miniscript::{MiniscriptKey, TranslatePk2};

use super::{Error, ScriptConstruction, ScriptSource};

pub trait DeriveLockScript {
    fn derive_lock_script<C: Verification>(
        &self,
        ctx: &Secp256k1<C>,
        child_index: UnhardenedIndex,
        descr_category: ConvertInfo,
    ) -> Result<LockScript, Error>;
}

impl DeriveLockScript for ScriptSource {
    fn derive_lock_script<C: Verification>(
        &self,
        ctx: &Secp256k1<C>,
        child_index: UnhardenedIndex,
        _: ConvertInfo,
    ) -> Result<LockScript, Error> {
        let ms = match &self.script {
            ScriptConstruction::Miniscript(ms) => ms.clone(),
            ScriptConstruction::MiniscriptPolicy(policy) => policy.compile()?,
            ScriptConstruction::ScriptTemplate(template) => {
                return Ok(Script::from(
                    template.translate_pk(ctx, child_index),
                )
                .into())
            }
        };

        let ms = ms.translate_pk2(|pk| {
            if pk.is_uncompressed() {
                return Err(Error::UncompressedKeyInSegWitContext);
            }
            Ok(pk.derive_public_key(ctx, child_index))
        })?;
        Ok(ms.encode().into())
    }
}
