//! a pack data file
use filebuffer::FileBuffer;
use std::{convert::TryInto, path::Path};

///
pub mod decode;
mod header;
pub use header::*;

mod init;
///
pub mod parse;
///
pub mod verify;

///
pub mod iter;
pub use iter::Iter;

use git_object::SHA1_SIZE;

/// A slice into a pack file denoting a pack entry.
///
/// An entry can be decoded into an object.
pub type EntrySlice = std::ops::Range<u64>;

/// Supported versions of a pack data file
#[derive(PartialEq, Eq, Debug, Hash, Ord, PartialOrd, Clone, Copy)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[allow(missing_docs)]
pub enum Version {
    V2,
    V3,
}

/// A pack data file
pub struct File {
    data: FileBuffer,
    path: std::path::PathBuf,
    version: Version,
    num_objects: u32,
}

/// Information about the pack data file itself
impl File {
    /// The pack data version of this file
    pub fn version(&self) -> Version {
        self.version
    }
    /// The number of objects stored in this pack data file
    pub fn num_objects(&self) -> u32 {
        self.num_objects
    }
    /// The length of all mapped data, including the pack header and the pack trailer
    pub fn data_len(&self) -> usize {
        self.data.len()
    }

    /// The position of the byte one past the last pack entry, or in other terms, the first byte of the trailing hash.
    pub fn pack_end(&self) -> usize {
        self.data.len() - SHA1_SIZE
    }

    /// The path to the pack data file on disk
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the pack data at the given slice if its range is contained in the mapped pack data
    pub fn entry_slice(&self, slice: EntrySlice) -> Option<&[u8]> {
        let entry_end: usize = slice.end.try_into().expect("end of pack fits into usize");
        let entry_start = slice.start as usize;
        self.data.get(entry_start..entry_end)
    }

    /// Returns the CRC32 of the pack data indicated by `pack_offset` and the `size` of the mapped data.
    ///
    /// _Note:_ finding the right size is only possible by decompressing
    /// the pack entry beforehand, or by using the (to be sorted) offsets stored in an index file.
    ///
    /// # Panics
    ///
    /// If `pack_offset` or `size` are pointing to a range outside of the mapped pack data.
    pub fn entry_crc32(&self, pack_offset: u64, size: usize) -> u32 {
        let pack_offset: usize = pack_offset.try_into().expect("pack_size fits into usize");
        git_features::hash::crc32(&self.data[pack_offset..pack_offset + size])
    }
}
