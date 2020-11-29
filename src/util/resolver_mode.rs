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

use std::num::ParseIntError;
use std::ops::Range;
use std::str::FromStr;

use lnpbp::secp256k1::rand::{rngs::ThreadRng, thread_rng, RngCore};

#[derive(Clone, PartialEq, Eq, Debug, Display, From, Error)]
#[display(doc_comments)]
pub enum ParseError {
    /// Unable to parse resolver mode directive: {0}
    #[from]
    InvalidInteger(ParseIntError),

    /// Unrecognized resolver mode name {0}
    UnrecognizedTypeName(String),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Display)]
pub enum ResolverModeType {
    #[display("while")]
    While,

    #[display("first{0}")]
    First(u32),

    #[display("random{0}")]
    Random(u32),
}

impl FromStr for ResolverModeType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some(s) = s.strip_prefix("first") {
            if s.is_empty() {
                ResolverModeType::First(1)
            } else {
                ResolverModeType::First(s.parse()?)
            }
        } else if let Some(s) = s.strip_prefix("random") {
            if s.is_empty() {
                ResolverModeType::Random(1)
            } else {
                ResolverModeType::Random(s.parse()?)
            }
        } else if s == "while" {
            ResolverModeType::While
        } else {
            Err(ParseError::UnrecognizedTypeName(s.to_owned()))?
        })
    }
}

impl ResolverModeType {
    pub fn count(self) -> usize {
        match self {
            ResolverModeType::While => 1usize,
            ResolverModeType::First(count) => count as usize,
            ResolverModeType::Random(count) => count as usize,
        }
    }

    pub fn range(self) -> Range<u32> {
        0u32..(self.count() as u32)
    }

    pub fn is_while(self) -> bool {
        self == ResolverModeType::While
    }
    pub fn is_random(self) -> bool {
        if let ResolverModeType::Random(_) = self {
            true
        } else {
            false
        }
    }
}

pub struct ResolverModeIter {
    mode: ResolverModeType,
    rand: ThreadRng,
    offset: u32,
}

impl IntoIterator for ResolverModeType {
    type Item = u32;
    type IntoIter = ResolverModeIter;

    fn into_iter(self) -> Self::IntoIter {
        ResolverModeIter {
            mode: self,
            rand: thread_rng(),
            offset: self.range().start,
        }
    }
}

impl Iterator for ResolverModeIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.mode.range().end {
            None
        } else {
            let index = if self.mode.is_random() {
                self.rand.next_u32()
            } else {
                self.offset
            };
            self.offset += 1;
            Some(index)
        }
    }
}
