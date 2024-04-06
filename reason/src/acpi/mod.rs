mod rsdp;

use rsdp::RootSystemDescriptorPointer;
use spin::MutexGuard;

use crate::boot::HHDM_OFFSET;
use crate::misc::log;
use crate::misc::utils::size;
use core::str;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct SdtHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oemid: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

pub fn initialize() {
    rsdp::initialize();

    let apic_header = find_header("APIC").unwrap();
}

fn validate_sdt_header(address: *const SdtHeader) -> bool {
    let size = unsafe { (*address).length };
    let address = address as *const u8;

    let mut checksum: u8 = 0;

    for i in 0..size {
        checksum = checksum.wrapping_add(unsafe { *address.add(i as usize) })
    }

    checksum == 0
}

pub fn find_header(signature: &str) -> Option<SdtHeader> {
    let rsdp = unsafe { rsdp::ROOT_SYSTEM_DESCRIPTOR_POINTER.lock() };

    type Rsdp = RootSystemDescriptorPointer;

    let (mut address, is_xsdt, sdt_ptr_size) = match *rsdp {
        Rsdp::V1 { address, .. } => (address, false, size!(u32)),
        Rsdp::V2 { address, .. } => (address, true, size!(u64)),
    };

    assert!(validate_sdt_header(address.cast::<SdtHeader>()));

    let rsdp_header = unsafe { address.cast::<SdtHeader>().read() };
    let number_of_headers = (rsdp_header.length as u64 - size!(SdtHeader)) / sdt_ptr_size;

    let mut address = address.as_addr() + size!(SdtHeader);

    for i in 0..number_of_headers {
        // The RSDT contains an array of pointers to other SystemDescriptorTables if we
        // are using xsdt (rsdt v2) then these pointers are 8 bytes (size_of::<u64>()) else the are
        // 4 bytes (size_of::<u32>()). I can't believe it took me hours to do this simple pointer
        // arithmetic :<<
        let header = unsafe {
            let addr = if is_xsdt {
                (address as *const u64).read() as u64 + HHDM_OFFSET
            } else {
                (address as *const u32).read() as u64 + HHDM_OFFSET
            };

            (addr as *const SdtHeader).read()
        };

        if signature.as_bytes() == &header.signature {
            return Some(header);
        }

        address += sdt_ptr_size
    }

    None
}
