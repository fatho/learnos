#[repr(C, packed)]
pub struct Header {
    pub total_size: u32,
    pub reserved: u32
}

#[repr(C, packed)]
pub struct Tag {
    pub tag_type: u32,
    pub size: u32
}
