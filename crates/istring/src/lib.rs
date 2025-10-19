// Copyright 2022 Sebastian Köln

// Licensed under the MIT license
// <LICENSE or http://opensource.org/licenses/MIT>

// The trait impls contains large chunks from alloc/string.rs,
// with the following copyright notice:

// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A replacement for String that allows storing strings of length up to sizeof<String>() - 1 without a heap allocation
//!
//! That means on 32bit machines: size_of::<IString>() == 12 bytes, inline capacity: 11 bytes
//! on 64bit machines: size_of::<IString>() == 24 bytes, inline capacity: 23 bytes

extern crate alloc;

#[macro_use]
mod common;

pub mod istring;
pub mod small;
pub mod ibytes;
pub mod tiny;

#[cfg(feature="serialize")]
use core::marker::PhantomData;

pub use crate::istring::IString;
pub use crate::ibytes::IBytes;
pub use crate::small::{SmallBytes, SmallString};
pub use crate::tiny::{TinyBytes, TinyString};

#[derive(Debug)]
pub struct FromUtf8Error<T> {
    bytes: T,
    error: core::str::Utf8Error,
}
impl<T: core::ops::Deref<Target=[u8]>> FromUtf8Error<T> {
    pub fn as_bytes(&self) -> &[u8] {
        &*self.bytes
    }
    pub fn into_bytes(self) -> T {
        self.bytes
    }
    pub fn utf8_error(&self) -> core::str::Utf8Error {
        self.error
    }
}


impl<T: core::fmt::Debug> core::fmt::Display for FromUtf8Error<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        self.error.fmt(f)
    }
}

impl<T: core::fmt::Debug> core::error::Error for FromUtf8Error<T> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        self.error.source()
    }
}


#[cfg(feature="serialize")]
use serde::{Serialize, Serializer, Deserialize, Deserializer, de::Visitor};

#[cfg(feature="serialize")]
use alloc::string::String;


#[cfg(feature="serialize")]
struct StringVisitor<T>(PhantomData<T>);

#[cfg(feature="serialize")]
impl<T> StringVisitor<T> {
    fn new() -> Self {
        StringVisitor(PhantomData)
    }
}

#[cfg(feature="serialize")]
impl<'de, T> Visitor<'de> for StringVisitor<T> where T: for<'a> From<&'a str> + From<String> {
    type Value = T;

    fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        write!(formatter, "a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {

        Ok(T::from(v))
    }
    fn visit_string<E>(self, v: alloc::string::String) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        
        Ok(T::from(v))
    }
}

#[cfg(feature="serialize")]
struct TinyStringVisitor;

#[cfg(feature="serialize")]
impl<'de> Visitor<'de> for TinyStringVisitor {
    type Value = TinyString;

    fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        write!(formatter, "a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {

        use serde::de::Error;
        TinyString::new(v).ok_or(Error::invalid_length(v.len(), &"less than 8 bytes"))
    }
}

#[cfg(feature="serialize")]
impl Serialize for IString {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>
    {
        self.as_str().serialize(serializer)
    }
}

#[cfg(feature="serialize")]
impl<'de> Deserialize<'de> for IString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_string(StringVisitor::<IString>::new())
    }
}

#[cfg(feature="serialize")]
impl Serialize for SmallString {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>
    {
        self.as_str().serialize(serializer)
    }
}

#[cfg(feature="serialize")]
impl<'de> Deserialize<'de> for SmallString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_string(StringVisitor::<SmallString>::new())
    }
}


#[cfg(feature="serialize")]
impl Serialize for TinyString {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>
    {
        self.as_str().serialize(serializer)
    }
}
#[cfg(feature="serialize")]
impl<'de> Deserialize<'de> for TinyString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_str(TinyStringVisitor)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use core::fmt::Write;
    use crate::istring::IString;

    #[test]
    fn test_thread() {
        let mut s = IString::from("Hello");
        write!(s, " world").unwrap();
        let s2 = thread::spawn(move || {
            let mut s = s;
            s += " from another thread!";
            s
        }).join().unwrap();
        assert_eq!(s2, "Hello world from another thread!");
    }

    #[test]
    fn test_misc_istring() {
        let p1 = "Hello World!";
        let p2 = "Hello World! .........xyz";
        let p3 = " .........xyz";
        
        let s1 = IString::from(p1);
        assert_eq!(s1, p1);
        
        let s2 = IString::from(p2);
        assert_eq!(s2, p2);
        
        let mut s3 = s1.clone();
        s3.push_str(p3);
        assert_eq!(s3, p2);
    }

    #[cfg(feature="size")]
    #[test]
    fn test_misc_smallstring() {
        let p1 = "Hello World!";
        let p2 = "Hello World! .........xyz";
        
        let s1 = crate::small::SmallString::from(p1);
        assert_eq!(s1, p1);
        
        let s2 = crate::small::SmallString::from(p2);
        assert_eq!(s2, p2);
    }

}
