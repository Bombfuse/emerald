use std::{fs::File, io::BufReader};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{Aseprite, EmeraldError};

pub fn word(buf_reader: &mut BufReader<File>) -> Result<u16, EmeraldError> {
    buf_reader.read_u16::<LittleEndian>().map_err(|e| e.into())
}
pub fn dword(buf_reader: &mut BufReader<File>) -> Result<u32, EmeraldError> {
    buf_reader.read_u32::<LittleEndian>().map_err(|e| e.into())
}

pub fn byte(buf_reader: &mut BufReader<File>) -> Result<u8, EmeraldError> {
    buf_reader.read_u8().map_err(|e| e.into())
}

pub fn short(buf_reader: &mut BufReader<File>) -> Result<i16, EmeraldError> {
    buf_reader.read_i16::<LittleEndian>().map_err(|e| e.into())
}

pub enum ChunkType {
    OldPalette04,
    OldPalette11,
    Layer,
    Cel,
    CelExtra,
    ColorProfile,
    ExternalFiles,
    Mask,
    Path,
    Tags,
    Palette,
    UserData,
    Slice,
    Tileset,
}
