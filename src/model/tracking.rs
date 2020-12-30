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

use lnpbp::bitcoin::util::base58;
use lnpbp::bitcoin::util::bip32::{self, ExtendedPrivKey, ExtendedPubKey};
use lnpbp::bp::bip32::Decode;
use lnpbp::bp::DerivationComponents;
use lnpbp::{bitcoin, secp256k1};

#[derive(Getters, Clone, PartialEq, Eq, Debug, StrictEncode, StrictDecode)]
pub struct TrackingAccount {
    pub name: String,
    pub key: TrackingKey,
}

impl TrackingAccount {
    pub fn details(&self) -> String {
        self.key.details()
    }

    pub fn count(&self) -> u32 {
        self.key.count()
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
    Display,
    StrictEncode,
    StrictDecode,
)]
#[display(TrackingKey::details)]
pub enum TrackingKey {
    SingleKey(secp256k1::PublicKey),
    HdKeySet(DerivationComponents),
}

impl TrackingKey {
    pub fn details(&self) -> String {
        match self {
            TrackingKey::SingleKey(ref pubkey) => pubkey.to_string(),
            TrackingKey::HdKeySet(ref keyset) => keyset.to_string(),
        }
    }

    pub fn count(&self) -> u32 {
        match self {
            TrackingKey::SingleKey(_) => 1,
            TrackingKey::HdKeySet(ref keyset) => keyset.count(),
        }
    }

    pub fn public_key(&self, index: u32) -> bitcoin::PublicKey {
        match self {
            TrackingKey::SingleKey(pk) => bitcoin::PublicKey {
                compressed: true,
                key: *pk,
            },
            TrackingKey::HdKeySet(keyset) => keyset.public_key(index),
        }
    }
}

// TODO: Consider moving the rest of the file to LNP/BP Core library

/// Extended public and private key processing errors
#[derive(Copy, Clone, PartialEq, Eq, Debug, Display, From, Error)]
#[display(doc_comments)]
pub enum Error {
    /// Error in BASE58 key encoding
    #[from(base58::Error)]
    Base58,

    /// A pk->pk derivation was attempted on a hardened key
    CannotDeriveFromHardenedKey,

    /// A child number was provided ({0}) that was out of range
    InvalidChildNumber(u32),

    /// Invalid child number format.
    InvalidChildNumberFormat,

    /// Invalid derivation path format.
    InvalidDerivationPathFormat,

    /// Unrecognized or unsupported extended key prefix (please check SLIP 32
    /// for possible values)
    UnknownSlip32Prefix,

    /// Failure in tust bitcoin library
    InteralFailure,
}

impl From<bip32::Error> for Error {
    fn from(err: bip32::Error) -> Self {
        match err {
            bip32::Error::CannotDeriveFromHardenedKey => {
                Error::CannotDeriveFromHardenedKey
            }
            bip32::Error::InvalidChildNumber(no) => {
                Error::InvalidChildNumber(no)
            }
            bip32::Error::InvalidChildNumberFormat => {
                Error::InvalidChildNumberFormat
            }
            bip32::Error::InvalidDerivationPathFormat => {
                Error::InvalidDerivationPathFormat
            }
            bip32::Error::Ecdsa(_) | bip32::Error::RngError(_) => {
                Error::InteralFailure
            }
        }
    }
}

pub trait FromSlip32 {
    fn from_slip32_str(s: &str) -> Result<Self, Error>
    where
        Self: Sized;
}

impl FromSlip32 for ExtendedPubKey {
    fn from_slip32_str(s: &str) -> Result<Self, Error> {
        const VERSION_MAGIC_XPUB: [u8; 4] = [0x04, 0x88, 0xB2, 0x1E];
        const VERSION_MAGIC_YPUB: [u8; 4] = [0x04, 0x9D, 0x7C, 0xB2];
        const VERSION_MAGIC_ZPUB: [u8; 4] = [0x04, 0xB2, 0x47, 0x46];
        const VERSION_MAGIC_YPUB_MULTISIG: [u8; 4] = [0x02, 0x95, 0xb4, 0x3f];
        const VERSION_MAGIC_ZPUB_MULTISIG: [u8; 4] = [0x02, 0xaa, 0x7e, 0xd3];

        const VERSION_MAGIC_TPUB: [u8; 4] = [0x04, 0x35, 0x87, 0xCF];
        const VERSION_MAGIC_UPUB: [u8; 4] = [0x04, 0x4A, 0x52, 0x62];
        const VERSION_MAGIC_VPUB: [u8; 4] = [0x04, 0x5F, 0x1C, 0xF6];
        const VERSION_MAGIC_UPUB_MULTISIG: [u8; 4] = [0x02, 0x42, 0x89, 0xef];
        const VERSION_MAGIC_VPUB_MULTISIG: [u8; 4] = [0x02, 0x57, 0x54, 0x83];

        let mut data = base58::from_check(s)?;

        let mut prefix = [0u8; 4];
        prefix.copy_from_slice(&data[0..4]);
        let slice = match prefix {
            VERSION_MAGIC_XPUB
            | VERSION_MAGIC_YPUB
            | VERSION_MAGIC_ZPUB
            | VERSION_MAGIC_YPUB_MULTISIG
            | VERSION_MAGIC_ZPUB_MULTISIG => VERSION_MAGIC_XPUB,

            VERSION_MAGIC_TPUB
            | VERSION_MAGIC_UPUB
            | VERSION_MAGIC_VPUB
            | VERSION_MAGIC_UPUB_MULTISIG
            | VERSION_MAGIC_VPUB_MULTISIG => VERSION_MAGIC_TPUB,

            _ => Err(Error::UnknownSlip32Prefix)?,
        };
        data[0..4].copy_from_slice(&slice);

        let xpub = ExtendedPubKey::decode(&data)?;

        Ok(xpub)
    }
}

impl FromSlip32 for ExtendedPrivKey {
    fn from_slip32_str(s: &str) -> Result<Self, Error> {
        const VERSION_MAGIC_XPRV: [u8; 4] = [0x04, 0x88, 0xAD, 0xE4];
        const VERSION_MAGIC_YPRV: [u8; 4] = [0x04, 0x9D, 0x78, 0x78];
        const VERSION_MAGIC_ZPRV: [u8; 4] = [0x04, 0xB2, 0x43, 0x0C];
        const VERSION_MAGIC_YPRV_MULTISIG: [u8; 4] = [0x02, 0x95, 0xb0, 0x05];
        const VERSION_MAGIC_ZPRV_MULTISIG: [u8; 4] = [0x02, 0xaa, 0x7a, 0x99];

        const VERSION_MAGIC_TPRV: [u8; 4] = [0x04, 0x35, 0x83, 0x94];
        const VERSION_MAGIC_UPRV: [u8; 4] = [0x04, 0x4A, 0x4E, 0x28];
        const VERSION_MAGIC_VPRV: [u8; 4] = [0x04, 0x5F, 0x18, 0xBC];
        const VERSION_MAGIC_UPRV_MULTISIG: [u8; 4] = [0x02, 0x42, 0x85, 0xb5];
        const VERSION_MAGIC_VPRV_MULTISIG: [u8; 4] = [0x02, 0x57, 0x50, 0x48];

        let mut data = base58::from_check(s)?;

        let mut prefix = [0u8; 4];
        prefix.copy_from_slice(&data[0..4]);
        let slice = match prefix {
            VERSION_MAGIC_XPRV
            | VERSION_MAGIC_YPRV
            | VERSION_MAGIC_ZPRV
            | VERSION_MAGIC_YPRV_MULTISIG
            | VERSION_MAGIC_ZPRV_MULTISIG => VERSION_MAGIC_XPRV,

            VERSION_MAGIC_TPRV
            | VERSION_MAGIC_UPRV
            | VERSION_MAGIC_VPRV
            | VERSION_MAGIC_UPRV_MULTISIG
            | VERSION_MAGIC_VPRV_MULTISIG => VERSION_MAGIC_TPRV,

            _ => Err(Error::UnknownSlip32Prefix)?,
        };
        data[0..4].copy_from_slice(&slice);

        let xprv = ExtendedPrivKey::decode(&data)?;

        Ok(xprv)
    }
}
