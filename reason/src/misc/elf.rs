use core::str::from_utf8_unchecked;
use core::{isize, str, usize};

use rustc_demangle::Demangle;

use crate::memory::VirtualAddress;
use crate::misc::utils::size;

use super::log;

#[repr(C)]
#[derive(Debug)]
pub struct Elf64Header {
    pub identification: [u8; 16],
    pub file_type: u16,
    pub architecture: u16,
    pub version: u32,
    pub program_address: u64,
    pub program_header_offset: u64,
    pub section_header_offset: u64,
    pub flags: u32,
    pub header_size: u16,
    pub program_header_size: u16,
    pub program_header_total: u16,
    pub section_header_size: u16,
    pub section_header_total: u16,
    pub string_table_index: u16,
}

#[repr(u32)]
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
    Loos = 0x60000000,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64SectionHeader {
    pub name_offset: u32,
    pub section_type: SectionHeaderType,
    pub flags: u64,
    pub address: u64,
    pub offset: u64,
    pub size: u64,
    pub link: u32,
    pub info: u32,
    pub address_align: u64,
    pub entry_size: u64,
}

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum SymbolType {
    NoType = 0,
    Object = 1,
    Func = 2,
    Section = 3,
    File = 4,
    Common = 5,
    TLS = 6,
    Unknown,
}

#[repr(u8)]
#[derive(Debug)]
pub enum SymbolBinding {
    Local = 0,
    Global = 1,
    Weak = 2,
}

#[repr(C)]
#[derive(Debug)]
pub struct Elf64Symbol {
    pub name_offset: u32,
    pub symbol_type: SymbolType,
    pub binding: SymbolBinding,
    pub shndx: u16,
    pub value: u64,
    pub size: u64,
}

pub struct Elf64 {
    address: VirtualAddress,
    header: Elf64Header,
}

use core::ffi::CStr;

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
            elf_header.identification[0..4],
            [0x7f, 0x45, 0x4c, 0x46],
            "invalid elf file"
        );

        assert_eq!(
            size!(Elf64SectionHeader),
            elf_header.section_header_size as u64
        );

        Self {
            address,
            header: elf_header,
        }
    }

    pub fn section_headers(&self) -> &[Elf64SectionHeader] {
        let address = self.address.as_addr() + self.header.section_header_offset;

        unsafe {
            core::slice::from_raw_parts(
                address as *const Elf64SectionHeader,
                self.header.section_header_total as usize,
            )
        }
    }

    pub fn symbol_table(&self) -> Elf64SectionHeader {
        *self
            .section_headers()
            .iter()
            .find(|header| header.section_type == SectionHeaderType::Symtab)
            .expect("Cannt find symbol table")
    }

    pub fn symbols(&self) -> Elf64SymbolIterator {
        let symbol_table = self.symbol_table();

        let total = (symbol_table.size / symbol_table.entry_size) as u16;
        let address = self.address.as_addr() + symbol_table.offset;

        Elf64SymbolIterator {
            address: address as *const Elf64Symbol,
            symbol_table_size: symbol_table.size,
            symbol_table_entry_size: symbol_table.entry_size,
            i: 0,
        }
    }

    pub fn get_name<'a>(&self, string_table_index: u32, string_table: u32) -> &'a str {
        let string_table = self
            .section_headers()
            .get(string_table as usize)
            .expect("Failed to find linked symbol table");

        let address = self.address.as_addr() + string_table.offset + string_table_index as u64;
        let string = unsafe { read_str(address as *const u8) };
        string
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
