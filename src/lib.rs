// Copyright (c) 2013-2015 Sandstorm Development Group, Inc. and contributors
// Licensed under the MIT License:
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

//! # Cap'n Proto Runtime Library
//!
//! [Cap'n Proto](http://kentonv.github.io/capnproto/) is an extremely efficient protocol for
//! sharing data and capabilities.
//!
//! The Rust implementation is split into three separate crates.
//!
//! Code generation is handled by [capnpc-rust](https://github.com/dwrensha/capnpc-rust).
//!
//! The present crate is the runtime library required by that generated code. It is hosted on Github
//! [here](https://github.com/dwrensha/capnproto-rust).
//!
//! [capnp-rpc-rust](https://github.com/dwrensha/capnp-rpc-rust) is an implementation of a
//! distributed object-capability layer.

#![feature(alloc, core, io, unsafe_destructor)]
#![allow(raw_pointer_derive)]

#![crate_name="capnp"]
#![crate_type = "lib"]

// reexports
pub use message::{MessageBuilder, BuilderOptions, MessageReader, ReaderOptions};
pub use message::MallocMessageBuilder;
pub use serialize::OwnedSpaceMessageReader;

pub mod any_pointer;
pub mod capability;
pub mod data;
pub mod data_list;
pub mod enum_list;
pub mod private;
pub mod io;
pub mod list_list;
pub mod message;
pub mod primitive_list;
pub mod serialize;
pub mod serialize_packed;
pub mod struct_list;
pub mod text;
pub mod text_list;
pub mod traits;

/// Eight bytes of memory with opaque interior.
///
/// This type is used to ensure that the data of a message is properly aligned.
#[derive(Copy)]
#[repr(C)]
pub struct Word {_unused_member : u64}

impl Word {
    /// Do this, but faster:
    /// `::std::iter::repeat(Word{ _unused_member : 0}).take(length).collect()`
    pub fn allocate_zeroed_vec(length : usize) -> ::std::vec::Vec<Word> {
        let mut result : ::std::vec::Vec<Word> = ::std::vec::Vec::with_capacity(length);
        unsafe {
            result.set_len(length);
            let p : *mut u8 = ::std::mem::transmute(result.as_mut_slice().as_mut_ptr());
            ::std::ptr::write_bytes(p, 0u8, length * ::std::mem::size_of::<Word>());
        }
        return result;
    }

    pub fn bytes_to_words<'a>(bytes : &'a [u8]) -> &'a [Word] {
        unsafe {
            ::std::slice::from_raw_parts(::std::mem::transmute(bytes.as_ptr()), bytes.len() / 8)
        }
    }

    pub fn words_to_bytes<'a>(words : &'a [Word]) -> &'a [u8] {
        unsafe {
            ::std::slice::from_raw_parts(::std::mem::transmute(words.as_ptr()), words.len() * 8)
        }
    }
}

/// Size of a message. Every generated struct has a method `.total_size()` that returns this.
#[derive(Copy, PartialEq)]
pub struct MessageSize {
    pub word_count : u64,

    /// Size of the capability table.
    pub cap_count : u32
}

impl MessageSize {
    pub fn plus_eq(&mut self, other : MessageSize) {
        self.word_count += other.word_count;
        self.cap_count += other.cap_count;
    }
}

/// An enum value or union discriminant that was not found among those defined in a schema.
#[derive(PartialEq, Copy, Debug)]
pub struct NotInSchema(pub u16);

/// Because messages are lazily validated, the return type of any method that reads a pointer field
/// must be wrapped in a Result.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Things that can go wrong when you read a message.
#[derive(Debug, PartialEq)]
pub enum Error {
    Decode { description : &'static str,
             detail : Option<String> },
    Io(std::io::Error),
}

impl Error {
    pub fn new_decode_error(description : &'static str, detail : Option<String>) -> Error {
        Error::Decode { description : description, detail : detail}
    }
}

impl ::std::error::FromError<::std::io::Error> for Error {
    fn from_error(err : ::std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl ::std::error::FromError<NotInSchema> for Error {
    fn from_error(NotInSchema(x) : NotInSchema) -> Error {
        Error::new_decode_error("Enum value or union descriminant not in schema.",
                                Some(format!("value : {}", x)))
    }
}

impl ::std::error::FromError<::std::str::Utf8Error> for Error {
    fn from_error(err : ::std::str::Utf8Error) -> Error {
        Error::new_decode_error("Text contains non-utf8 data.",
                                Some(format!("{:?}", err)))
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, fmt : &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        match *self {
            Error::Decode { ref description, detail : Some(ref detail) } => {
                write!(fmt, "{} {}", description, detail)
            },
            Error::Decode { ref description, .. } => write!(fmt, "{}", description),
            Error::Io(ref io) => io.fmt(fmt),
        }
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Decode { ref description, .. } => description,
            Error::Io(ref io) => ::std::error::Error::description(io),
        }
    }
    fn cause(&self) -> Option<&::std::error::Error> {
        match *self {
            Error::Decode { .. } => None,
            Error::Io(ref io) => io.cause(),
        }
    }
}