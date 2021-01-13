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

mod descriptors;
mod document;
pub mod operation;
mod tracking;
mod utxo;

pub use descriptors::{
    DescriptorContent, DescriptorGenerator, Error as DescriptorError,
    SourceType,
};
pub use document::{Document, Error, Profile, ResolverError};
pub use tracking::{
    Error as Slip32Error, FromSlip32, TrackingAccount, TrackingKey,
};
pub use utxo::UtxoEntry;
