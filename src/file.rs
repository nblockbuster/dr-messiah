use binrw::{binread, BinRead};

use super::{material, model};
#[binread]
#[derive(Debug, Clone)]
#[br(magic = b".MESSIAH")]
pub struct MessiahHeader {
    pub data: MessiahTypes,
}

#[derive(BinRead, Debug, Clone)]
pub enum MessiahTypes {
    #[br(magic = 0x4_u32)]
    Material(material::MaterialHeader),
    #[br(magic = 0x8_u32)]
    Model(model::ModelHeader),
}
