use core::str::from_utf8_unchecked;
use core::{isize, str, usize};

use crate::memory::VirtualAddress;
use crate::misc::utils::size;

use super::log;

#[repr(C)]
#[derive(Debug)]
pub struct Elf64Header {
    /// Magic number and other info
    pub e_ident: [u8; 16],
    /// Object file type
    pub e_type: u16,
    /// Architecture
    pub e_machine: u16,
    /// Object file version
    pub e_version: u32,
    /// Entry point virtual address
    pub e_entry: u64,
    /// Program header table file offset
    pub e_phoff: u64,
    /// Section header table file offset
    pub e_shoff: u64,
    /// Processor-specific flags
    pub e_flags: u32,
    /// ELF header size in bytes
    pub e_ehsize: u16,
    /// Program header table entry size
    pub e_phentsize: u16,
    /// Program header table entry count
    pub e_phnum: u16,
    /// Section header table entry size
    pub e_shentsize: u16,
    /// Section header table entry count
    pub e_shnum: u16,
    /// Section header string table index
    pub e_shstrndx: u16,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SectionHeaderType {
    Null = 0x0,
    Progbits = 0x1,
    Symtab = 0x2,
    Strtab = 0x3,
    Rela = 0x4,
    Hash = 0x5,
    Dynamic = 0x6,
    Note = 0x7,
    Nobits = 0x8,
    Rel = 0x09,
    Shlib = 0xA,
    Dynsym = 0xB,
    InitArray = 0xE,
    FiniArray = 0xF,
    PreinitArray = 0x10,
    Group = 0x11,
    SymtabShndx = 0x12,
    Num = 0x13,
    LOOS = 0x60000000,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64SectionHeader {
    /// Section name (string tbl index)
    pub sh_name: u32,
    /// Section type
    pub sh_type: u32,
    /// Section flags
    pub sh_flags: u64,
    /// Section virtual addr at executtion
    pub sh_addr: u64,
    /// Section file offset
    pub sh_offset: u64,
    /// Section size in bytes
    pub sh_size: u64,
    /// Link to another section
    pub sh_link: u32,
    /// Additional another section
    pub sh_info: u32,
    /// Section alignment
    pub sh_addralign: u64,
    /// Entry size if section holds table
    pub sh_entsize: u64,
}

#[derive(Debug, PartialEq)]
pub enum SymbolType {
    NoType,
    Object,
    Func,
    Section,
    File,
    Common,
    TLS,
    Num,
    LOOS,
    HIOS,
    LOPROC,
    HIPROC,
    Unknown,
}

#[derive(Debug)]
pub enum SymbolBinding {
    Local,
    Global,
    Weak,
    Num,
    LOOS,
    HIOS,
    LOPROC,
    HIPROC,
    Unknown,
}

pub enum SymbolVisibility {
    Default,
    Internal,
    Hidden,
    Protected,
    Unknown,
}

#[repr(C)]
#[derive(Debug)]
pub struct Elf64Symbol {
    /// Symbol name (string tbl index)
    pub st_name: u32,
    /// Symbol type and binding
    pub st_info: u8,
    /// Symbol visibility
    pub st_other: u8,
    /// Section index
    pub st_shndx: u16,
    /// Symbol value
    pub st_value: u64,
    /// Symbol size
    pub st_size: u64,
}

pub struct Elf64 {
    address: VirtualAddress,
    header: Elf64Header,
}

unsafe fn read_str<'a>(input: *const u8) -> &'a str {
    let mut byte = input.offset(0);
    let mut i = 0;

    while !(*byte == 0) {
        i += 1;
        byte = unsafe { input.offset(i as isize) };
    }

    core::str::from_utf8_unchecked(core::slice::from_raw_parts(input, i))
}

impl Elf64 {
    pub fn new(address: VirtualAddress) -> Self {
        let elf_header = unsafe { (address.as_addr() as *const Elf64Header).read() };

        assert_eq!(
            elf_header.e_ident[0..4],
            [0x7f, 0x45, 0x4c, 0x46],
            "invalid elf file"
        );

        assert_eq!(size!(Elf64SectionHeader), elf_header.e_shentsize as u64);

        Self {
            address,
            header: elf_header,
        }
    }

    pub fn section_headers(&self) -> &[Elf64SectionHeader] {
        let address = self.address.as_addr() + self.header.e_shoff;

        unsafe {
            core::slice::from_raw_parts(
                address as *const Elf64SectionHeader,
                self.header.e_shnum as usize,
            )
        }
    }

    pub fn symbol_table(&self) -> Elf64SectionHeader {
        *self
            .section_headers()
            .iter()
            .find(|header| header.sh_type == SectionHeaderType::Symtab as u32)
            .expect("Cannt find symbol table")
    }

    pub fn symbols(&self) -> Elf64SymbolIterator {
        let symbol_table = self.symbol_table();

        let total = (symbol_table.sh_size / symbol_table.sh_entsize) as u16;
        let address = self.address.as_addr() + symbol_table.sh_offset;

        Elf64SymbolIterator {
            address: address as *const Elf64Symbol,
            symbol_table_size: symbol_table.sh_size,
            symbol_table_entry_size: symbol_table.sh_entsize,
            i: 0,
        }
    }

    pub fn get_name<'a>(&self, string_table_index: u32, string_table: u32) -> &'a str {
        let string_table = self
            .section_headers()
            .get(string_table as usize)
            .expect("Failed to find linked symbol table");

        let address = self.address.as_addr() + string_table.sh_offset + string_table_index as u64;
        let string = unsafe { read_str(address as *const u8) };
        string
    }
}

impl Elf64Symbol {
    pub const fn r#type(&self) -> SymbolType {
        match (self.st_info & 0xf) {
            0 => SymbolType::NoType,
            1 => SymbolType::Object,
            2 => SymbolType::Func,
            3 => SymbolType::Section,
            4 => SymbolType::File,
            5 => SymbolType::Common,
            6 => SymbolType::TLS,
            7 => SymbolType::Num,
            10 => SymbolType::LOOS,
            12 => SymbolType::HIOS,
            13 => SymbolType::LOPROC,
            14 => SymbolType::HIPROC,
            _ => SymbolType::Unknown,
        }
    }

    pub const fn binding(&self) -> SymbolBinding {
        match (self.st_info >> 4) {
            0 => SymbolBinding::Local,
            1 => SymbolBinding::Global,
            2 => SymbolBinding::Weak,
            3 => SymbolBinding::Num,
            10 => SymbolBinding::LOOS,
            12 => SymbolBinding::HIOS,
            13 => SymbolBinding::LOPROC,
            15 => SymbolBinding::HIPROC,
            _ => SymbolBinding::Unknown,
        }
    }

    pub const fn within(&self, address: VirtualAddress) -> bool {
        let address = address.as_addr();
        address >= self.st_value && address < self.st_value + self.st_size
    }

    pub const fn visibility(&self) -> SymbolVisibility {
        match (self.st_other & 0x03) {
            0 => SymbolVisibility::Default,
            1 => SymbolVisibility::Internal,
            2 => SymbolVisibility::Hidden,
            3 => SymbolVisibility::Protected,
            _ => SymbolVisibility::Unknown,
        }
    }
}

pub struct Elf64SectionHeaderIterator {
    base_address: *const Elf64SectionHeader,
    index: u16,
    total: u16,
}

pub struct Elf64SymbolIterator {
    address: *const Elf64Symbol,
    symbol_table_entry_size: u64,
    symbol_table_size: u64,
    i: isize,
}
impl Iterator for Elf64SectionHeaderIterator {
    type Item = Elf64SectionHeader;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.total {
            return None;
        }

        let header = unsafe { self.base_address.offset(self.index as isize) };
        self.index += 1;

        unsafe { Some(header.read()) }
    }
}

impl Iterator for Elf64SymbolIterator {
    type Item = Elf64Symbol;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i as u64 == self.symbol_table_size {
            return None;
        }

        let symbol = unsafe { self.address.byte_offset(self.i).read() };

        self.i += self.symbol_table_entry_size as isize;
        Some(symbol)
    }
}
