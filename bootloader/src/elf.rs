#[repr(u8)]
#[allow(dead_code)]
pub enum ElfClass {
    Bit32 = 1,
    Bit64 = 2
}

#[allow(dead_code)]

pub enum Endianness {
    Little = 1,
    Big = 2
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ElfType {
    Relocatable = 1,
    Executable = 2,
    Shared = 3,
    Core = 4
}

#[allow(dead_code)]
pub enum ElfInstructionSet {
    None = 0x0,
    SPARC = 0x02,
    X86 = 0x03,
    MIPS = 0x08,
    PowerPC = 0x14,
    ARM = 0x28,
    SuperH = 0x2A,
    IA64 = 0x32,
    X86_64 = 0x3E,
    AARCH64 = 0xB7,
    RISCV = 0xF3
}

#[allow(dead_code)]
pub enum ProgramHeaderSectionType {
    Null = 0,
    Load = 1,
    Dynamic = 2,
    Interp = 3,
    Note = 4
}

#[repr(C, packed)]
pub struct ElfIdentifier {
    pub magic: [u8;4],
    pub class: u8,
    pub endianness: u8,
    pub header_version: u8,
    pub abi: u8,
    _padding: [u8;8]
}

impl ElfIdentifier {
    pub fn new() -> ElfIdentifier {
        ElfIdentifier { magic: [0;4], class: 0, endianness: 0, header_version: 0, abi: 0, _padding: [0;8]}
    }
}

#[repr(C, packed(8))]
pub struct ElfHeader {
    pub identifier: ElfIdentifier,
    pub elf_type: u16,
    pub instruction_set: u16,
    pub elf_version: u32,
    pub entry_offset: u64,
    pub program_header_table_offset: u64,
    pub section_header_table_offset: u64,
    pub flags: u32,
    pub header_size: u16,
    pub program_header_entry_size: u16,
    pub program_header_entry_count: u16,
    pub section_header_entry_size: u16,
    pub section_header_entry_count: u16,
    pub section_header_string_index: u16
}

#[repr(C)]
pub struct ProgramHeader {
    pub segment_type: u32,
    pub flags: u32,
    pub data_offset: u64,
    pub virtual_address: u64,
    pub physical_address: u64,
    pub file_size: u64,
    pub mem_size: u64,
    pub alignment: u64
}