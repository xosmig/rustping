// more efficient implementations are possible with use of unsafe::transmute
// and from_be / to_be standard functions

pub fn be_to_u16(be: &[u8]) -> u16 {
    (be[0] as u16) + ((be[1] as u16) << 8)
}

pub fn u16_to_be(val: u16) -> [u8; 2] {
    [(val & 0xFF) as u8, (val >> 8) as u8]
}

pub fn u32_to_be(val: u32) -> [u8; 4] {
    [(val & 0xFF) as u8, ((val >> 8) & 0xFF) as u8, ((val >> 16) & 0xFF) as u8, (val >> 24) as u8]
}
