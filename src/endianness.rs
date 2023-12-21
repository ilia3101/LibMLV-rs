/* Endianness */

macro_rules! impl_little_endian {
    ($t: ty, $name: ident) => {
        #[repr(transparent)]
        #[derive(Debug,Clone,Copy,PartialEq)]
        #[allow(non_camel_case_types)]
        pub struct $name ($t);

        impl $name {
            #[inline(always)] pub fn get(self) -> $t { <$t>::from_le(self.0) }
            #[inline(always)] pub fn set(&mut self, v: $t) { self.0 = v.to_le() }
            #[inline(always)] pub fn new(v: $t) -> $name { $name(v.to_le()) }
        }

        impl PartialEq<$t> for $name {
            #[inline(always)] fn eq(&self, other: &$t) -> bool { self.get() == *other }
        }

        impl Into<$name> for $t {
            #[inline(always)] fn into(self) -> $name { $name(self.to_le()) }
        }

        impl core::fmt::Display for $name {
            #[inline] fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { self.0.fmt(f) }
        }
    };
}

impl_little_endian!{u8, u8le}
impl_little_endian!{i8, i8le}
impl_little_endian!{u16, u16le}
impl_little_endian!{i16, i16le}
impl_little_endian!{u32, u32le}
impl_little_endian!{i32, i32le}
impl_little_endian!{u64, u64le}
impl_little_endian!{i64, i64le}