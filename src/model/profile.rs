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

use amplify::internet::InetSocketAddr;
use lnpbp::bp::Chain;
use lnpbp::lnp::{NodeAddr, RemoteNodeAddr};
// use rgb::fungible;

use super::{operation, TrackingAccount, UtxoEntry};

#[derive(Clone, PartialEq, Eq, Debug, Default, StrictEncode, StrictDecode)]
pub struct Profile {
    pub name: String,
    pub description: Option<String>,
    pub tracking: Vec<TrackingAccount>,
    pub utxo_cache: Vec<UtxoEntry>,
    // pub assets_cache: Vec<fungible::Asset>,
    pub history: Vec<operation::LogEntry>,
    pub settings: Settings,
}

#[derive(Clone, PartialEq, Eq, Debug, Display, StrictEncode, StrictDecode)]
pub enum ChainResolver {
    #[display("bitcoinCore({0})")]
    BitcoinCore(InetSocketAddr),
    #[display("electrum({0})")]
    Electrum(InetSocketAddr),
    #[display("bpNode({0})")]
    BpNode(NodeAddr),
}

impl Default for ChainResolver {
    fn default() -> Self {
        ChainResolver::Electrum(
            "31.14.40.18:60000"
                .parse()
                .expect("Predefined address always parses"),
        )
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default, StrictEncode, StrictDecode)]
pub struct Settings {
    pub chain: Chain,
    pub resolver: ChainResolver,
    pub bifrost: Option<RemoteNodeAddr>,
}
