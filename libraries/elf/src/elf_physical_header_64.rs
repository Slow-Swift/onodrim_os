#[derive(PartialEq, Debug)]
pub enum ElfPhysicalType {
    Null,
    Load,
    Dynamic,
    Interp,
    Note,
    Shlib,
    Phdr,
    Tls,
    Num,
    LoOs,
    GnuEhFrame,
    GnuStack,
    GnuRelRO,
    GnuProperty,
    SunWBss,
    SunWStack,
    HiSunW,
    LoProc,
    HiProc,
}

#[repr(C)]
pub struct ElfPhysicalHeader64 {
    p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

impl ElfPhysicalHeader64 {
    pub fn p_type(&self) -> ElfPhysicalType {
        match self.p_type {
            1 => ElfPhysicalType::Load,
            2 => ElfPhysicalType::Dynamic,
            3 => ElfPhysicalType::Interp,
            4 => ElfPhysicalType::Note,
            5 => ElfPhysicalType::Shlib,
            6 => ElfPhysicalType::Phdr,
            7 => ElfPhysicalType::Tls,
            8 => ElfPhysicalType::Num,
            0x60000000 => ElfPhysicalType::LoOs,
            0x6474e550 => ElfPhysicalType::GnuEhFrame,
            0x6474e551 => ElfPhysicalType::GnuStack,
            0x6474e552 => ElfPhysicalType::GnuRelRO,
            0x6474e553 => ElfPhysicalType::GnuProperty,
            0x6ffffffa => ElfPhysicalType::SunWBss,
            0x6ffffffb => ElfPhysicalType::SunWStack,
            0x6fffffff => ElfPhysicalType::HiSunW,
            0x70000000 => ElfPhysicalType::LoProc,
            0x7fffffff => ElfPhysicalType::HiProc,
            _ => ElfPhysicalType::Null,
        }
    }
}

impl Default for ElfPhysicalHeader64 {
    fn default() -> Self {
        ElfPhysicalHeader64 {
            p_type: 0,
            p_flags: 0,
            p_offset: 0,
            p_vaddr: 0,
            p_paddr: 0,
            p_filesz: 0,
            p_memsz: 0,
            p_align: 0,
        }
    }
}