use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, BufRead};

pub mod classic;

pub trait Packet<T> {
    fn from(buffer: T) -> Self;
    fn into(self) -> Vec<u8>;
}