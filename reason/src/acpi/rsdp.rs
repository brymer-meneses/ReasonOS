use crate::boot::{self, HHDM_OFFSET, RSDP_REQUEST};
use crate::memory::IntoAddress;
use crate::memory::VirtualAddress;
use crate::misc::log;
use crate::misc::utils::{size, OnceLock};

use core::str;

pub enum RootSystemDescriptorPointer {
    V1 { address: VirtualAddress, size: u64 },
    V2 { address: VirtualAddress, size: u64 },
}

impl RootSystemDescriptorPointer {
    fn from_address(address: VirtualAddress) -> Option<Self> {
        #[repr(C, packed)]
        struct RsdpV1 {
            signature: [u8; 8],
            checksum: u8,
            oemid: [u8; 6],
            revision: u8,
            rsdt_address: u32,
        }

        #[repr(C, packed)]
        struct RsdpV2 {
            signature: [u8; 8],
            checksum: u8,
            oemid: [u8; 6],
            revision: u8,
            rsdt_address: u32,
            length: u32,
            xsdt_address: u64,
            extended_checksum: u8,
            reserved: [u8; 3],
        }

        let rsdp = unsafe { address.cast::<RsdpV2>().read() };

        assert_eq!(str::from_utf8(&rsdp.signature).unwrap(), "RSD PTR ");

        let is_xsdt = {
            if rsdp.revision == 0 {
                false
            } else {
                true
            }
        };

        let size = if is_xsdt {
            size!(RsdpV2)
        } else {
            size!(RsdpV1)
        };

        type Rsdp = RootSystemDescriptorPointer;

        let checksum = (0..size as usize)
            .map(|i| unsafe { address.as_ptr().add(i) })
            .fold(0, |acc: u32, byte| unsafe { acc + (*byte) as u32 });

        assert_eq!(checksum & 0xFF, 0);

        let hhdm_offset = unsafe { HHDM_OFFSET };

        if is_xsdt {
            let address = rsdp.xsdt_address as u64;
            return Some(Rsdp::V2 {
                address: VirtualAddress::new(address + hhdm_offset),
                size: size!(RsdpV2),
            });
        } else {
            let address = rsdp.rsdt_address as u64;
            return Some(Rsdp::V1 {
                address: VirtualAddress::new(address + hhdm_offset),
                size: size!(RsdpV1),
            });
        }
    }
}

pub static mut ROOT_SYSTEM_DESCRIPTOR_POINTER: OnceLock<RootSystemDescriptorPointer> =
    OnceLock::new();

pub fn initialize() {
    let address = (boot::RSDP_REQUEST
        .get_response()
        .get()
        .expect("Failed to get RSDP Response")
        .address
        .as_ptr()
        .unwrap() as u64)
        .as_virtual();

    let rsdp = RootSystemDescriptorPointer::from_address(address).unwrap();

    unsafe { ROOT_SYSTEM_DESCRIPTOR_POINTER.set(rsdp) };

    log::info!("Initialized RSDP");
}
