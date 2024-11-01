
use binrw::BinRead;

use crate::model;

#[derive(BinRead, Debug, Clone)]
#[br(magic = b".MESSIAH")]
pub struct MessiahHeader {
    pub type_: MessiahTypes,
}

#[derive(BinRead, Debug, Clone)]
#[repr(u32)]
pub enum MessiahTypes {
    Material = 0x4,
    Model(model::ModelHeader) = 0x8
}
