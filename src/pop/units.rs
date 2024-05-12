use std::io::Read;
use core::mem::size_of;

use crate::pop::types::{BinDeserializer, from_reader};

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct UnitRaw {
    pub unit_type: u8,
    pub unit_class: u8,
    tribe_index: u8,
    loc_x: u16,
    loc_y: u16,
    f1: u32,
    f2: u16,
    f3: u16,
    fd: [u8; 40],
}

impl BinDeserializer for UnitRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<UnitRaw, {size_of::<UnitRaw>()}, R>(reader)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct TribeConfigRaw {
    pub data: [u8; 16],
}

impl BinDeserializer for TribeConfigRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<TribeConfigRaw, {size_of::<TribeConfigRaw>()}, R>(reader)
    }
}

/******************************************************************************/
