extern crate byteorder;

use byteorder::{BigEndian, LittleEndian};

use std::fmt;
use std::io::{self, Cursor};
use std::string::FromUtf8Error;
use std::io::prelude::*;

// ReadDirectorValues

pub trait ReadDirectorValues {
    fn read_string(&mut self, length: usize) -> Result<String, ReadStringError>;
    fn read_fourcc<B: ByteOrder>(&mut self) -> Result<String, ReadStringError>;
}

impl<'a> ReadDirectorValues for Cursor<&'a [u8]> {
    fn read_string(&mut self, length: usize) -> Result<String, ReadStringError> {
        let mut buf = vec![0; length];
        self.read_exact(&mut buf)?;
        let string = String::from_utf8(buf)?;
        Ok(string)
    }

    fn read_fourcc<B: ByteOrder>(&mut self) -> Result<String, ReadStringError> {
        let mut buf = vec![0; 4];
        self.read_exact(&mut buf)?;
        let fourcc = B::read_fourcc(buf)?;
        Ok(fourcc)
    }
}

// ByteOrder extension

pub trait ByteOrder: byteorder::ByteOrder {
    fn read_fourcc(buf: Vec<u8>) -> Result<String, FromUtf8Error>;
}

impl ByteOrder for BigEndian {
    fn read_fourcc(buf: Vec<u8>) -> Result<String, FromUtf8Error> {
        String::from_utf8(buf)
    }
}

impl ByteOrder for LittleEndian {
    fn read_fourcc(buf: Vec<u8>) -> Result<String, FromUtf8Error> {
        String::from_utf8(vec![buf[3], buf[2], buf[1], buf[0]])
    }
}

// ReadStringError

pub enum ReadStringError {
    IOError(io::Error),
    Utf8Error(FromUtf8Error),
}

impl From<io::Error> for ReadStringError {
    fn from(e: io::Error) -> ReadStringError {
        ReadStringError::IOError(e)
    }
}

impl From<FromUtf8Error> for ReadStringError {
    fn from(e: FromUtf8Error) -> ReadStringError {
        ReadStringError::Utf8Error(e)
    }
}

impl fmt::Debug for ReadStringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReadStringError::IOError(ref e) => e.fmt(f),
            ReadStringError::Utf8Error(ref e) => e.fmt(f),
        }
    }
}
