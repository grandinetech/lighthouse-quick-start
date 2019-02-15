use crate::slot_epoch::{Epoch, Slot};
use crate::test_utils::TestRandom;
use rand::RngCore;
use serde_derive::Serialize;
use ssz::{hash, ssz_encode, Decodable, DecodeError, Encodable, SszStream, TreeHash};
use std::cmp::{Ord, Ordering};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, Sub, SubAssign};

/// Beacon block height, effectively `Slot/GENESIS_START_BLOCK`.
#[derive(Eq, Debug, Clone, Copy, Default, Serialize)]
pub struct SlotHeight(u64);

impl_common!(SlotHeight);
impl_into_u32!(SlotHeight); // SlotHeight can be converted to u32

impl SlotHeight {
    pub fn new(slot: u64) -> SlotHeight {
        SlotHeight(slot)
    }

    pub fn slot(self, genesis_slot: Slot) -> Slot {
        Slot::from(self.0.saturating_add(genesis_slot.as_u64()))
    }

    pub fn epoch(self, genesis_slot: u64, epoch_length: u64) -> Epoch {
        Epoch::from(self.0.saturating_add(genesis_slot) / epoch_length)
    }

    pub fn max_value() -> SlotHeight {
        SlotHeight(u64::max_value())
    }
}
