// Descriptor wallet library extending bitcoin & miniscript functionality
// by LNP/BP Association (https://lnp-bp.org)
// Written in 2020-2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the Apache-2.0 License
// along with this software.
// If not, see <https://opensource.org/licenses/Apache-2.0>.

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
