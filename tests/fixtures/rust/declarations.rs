//! Declares items with docs,
//! headers and exports.

/// Builds a value
/// across two lines.
#[inline]
pub fn build() -> u32 { // trailing note
    1
}

pub(crate) fn shared() {}

fn private() {}

#[doc = "Holds a value."]
pub struct Holder(u32);

impl Holder {
    /// Reads the value.
    pub fn get(&self) -> u32 {
        self.0
    }
}
