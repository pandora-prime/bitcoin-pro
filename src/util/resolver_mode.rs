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

use std::num::ParseIntError;
use std::ops::Range;
use std::str::FromStr;

use bitcoin::secp256k1::rand::{rngs::ThreadRng, thread_rng, RngCore};
use wallet::hd::{SegmentIndexes, UnhardenedIndex};

#[derive(Clone, PartialEq, Eq, Debug, Display, From, Error)]
#[display(doc_comments)]
pub enum ParseError {
    /// Unable to parse resolver mode directive: {0}
    #[from]
    InvalidInteger(ParseIntError),

    /// The actual value of the used index corresponds to a hardened index,
    /// which can't be used in the current context
    HardenedIndex,

    /// Unrecognized resolver mode name {0}
    UnrecognizedTypeName(String),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Display)]
pub enum ResolverModeType {
    #[display("while")]
    While,

    #[display("first{0}")]
    First(UnhardenedIndex),

    #[display("random{0}")]
    Random(UnhardenedIndex),
}

impl FromStr for ResolverModeType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some(s) = s.strip_prefix("first") {
            if s.is_empty() {
                ResolverModeType::First(UnhardenedIndex::one())
            } else {
                ResolverModeType::First(
                    UnhardenedIndex::from_index(u32::from_str(s)?)
                        .map_err(|_| ParseError::HardenedIndex)?,
                )
            }
        } else if let Some(s) = s.strip_prefix("random") {
            if s.is_empty() {
                ResolverModeType::Random(UnhardenedIndex::one())
            } else {
                ResolverModeType::Random(
                    UnhardenedIndex::from_index(u32::from_str(s)?)
                        .map_err(|_| ParseError::HardenedIndex)?,
                )
            }
        } else if s == "while" {
            ResolverModeType::While
        } else {
            return Err(ParseError::UnrecognizedTypeName(s.to_owned()));
        })
    }
}

impl ResolverModeType {
    pub fn count(self) -> usize {
        match self {
            ResolverModeType::While => 1usize,
            ResolverModeType::First(count) => count.first_index() as usize,
            ResolverModeType::Random(count) => count.first_index() as usize,
        }
    }

    pub fn range(self) -> Range<u32> {
        0u32..(self.count() as u32)
    }

    pub fn is_while(self) -> bool {
        self == ResolverModeType::While
    }
    pub fn is_random(self) -> bool {
        matches!(self, ResolverModeType::Random(_))
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
