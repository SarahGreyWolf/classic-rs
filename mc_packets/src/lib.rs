//! # MC Packets
//! Packets used by minecraft for communicating between server and client
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, BufRead};

pub mod classic;

/// # Packet
pub trait Packet<T> {
    /// Get a Packet from a buffer
    fn from(buffer: T) -> Self;
    /// Create a buffer of Vec<u8> from a packet
    fn into(&self) -> Vec<u8>;
    /// Returns the byte length of the packet
    fn size(id: u8) -> usize;
}