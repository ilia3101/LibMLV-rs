#[cfg(feature = "jpeg2000")]
pub mod jpeg2000;
pub mod lj92;
pub mod packed;

pub use packed::*;

use lj92::{Lj92, Lj92Error};

#[inline]
pub fn decode_lj92(data: &[u8], out: &mut [u16]) -> Result<(), Lj92Error> {
    match Lj92::open(data) {
        Ok(mut lj92) => lj92.decode(out, 0, None),
        Err(err) => Err(err),
    }
}
