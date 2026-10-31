//! The plan's index from component to position: an open-addressing hash table
//! whose probe *starts* at the hash of the type name and *decides* by `TypeId`.
//!
//! The name hash is only a hint for where to look, so a collision or two
//! types with the same `type_name` cost an extra probe and nothing else: a
//! slot is a match only if the `TypeId` stored for it is equal, otherwise the
//! probe moves on, and two such types become two separate entries.
//!
//! Why not just scan the plan's `TypeId`s? In constant evaluation a `TypeId`
//! can be compared with `==` but not ordered or hashed (see `identity`), and a
//! scan costs about 460,000 comparisons for the 397-component benchmark, which
//! was +1.2 s of compile time. With the table a lookup is about one comparison.

use std::any::TypeId;

use crate::plan::MAX_COMPONENTS;

/// Slots of the table: a power of two, at least twice the components.
const CAP: usize = MAX_COMPONENTS * 2;

/// Position + 1 of the component in each slot (0 = empty). The components'
/// `TypeId`s live in the plan, by position; the table only holds positions.
#[doc(hidden)]
pub struct Table {
    slots: [u32; CAP],
}

impl Table {
    pub(crate) const fn new() -> Table {
        Table { slots: [0; CAP] }
    }

    /// The slot of `id`: the one holding it, or else the empty slot where it
    /// would go. Probing starts at `hash` and passes slots of other types.
    pub(crate) const fn slot(&self, ids: &[TypeId], hash: u64, id: TypeId) -> usize {
        let mut s = (hash as usize) & (CAP - 1);
        loop {
            let held = self.slots[s];
            if held == 0 || ids[held as usize - 1] == id {
                return s;
            }
            s = (s + 1) & (CAP - 1);
        }
    }

    /// The position held in slot `s`, if any.
    pub(crate) const fn at(&self, s: usize) -> Option<usize> {
        match self.slots[s] {
            0 => None,
            held => Some(held as usize - 1),
        }
    }

    /// Record that position `pos` is in (empty) slot `s`.
    pub(crate) const fn put(&mut self, s: usize, pos: usize) {
        self.slots[s] = pos as u32 + 1;
    }
}
