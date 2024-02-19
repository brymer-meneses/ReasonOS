use crate::{
    boot,
    memory::VirtualAddress,
    misc::{log, utils::size, utils::OnceLock},
};

use core::str;

#[repr(C, packed)]
pub struct Rsdp {
    signature: [u8; 8],
    checksum: u8,
    oemid: [u8; 6],
    revision: u8,
    rstd_address: u32,
}

#[repr(C, packed)]
pub struct Xsdp {
    signature: [u8; 8],
    checksum: u8,
    oemid: [u8; 6],
    revision: u8,
    rstd_address: u32,

    length: u32,
    xstd_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

pub enum RootSystemDescriptorPointer {
    V1(Rsdp),
    V2(Xsdp),
}

impl RootSystemDescriptorPointer {
    pub fn new(address: VirtualAddress) -> RootSystemDescriptorPointer {
        let rsdp = unsafe { address.cast::<Xsdp>().read() };

        assert_eq!(str::from_utf8(&rsdp.signature).unwrap(), "RSD PTR ");

        let size = match rsdp.revision >= 2 && rsdp.xstd_address != 0 {
            true => size!(Xsdp),
            false => size!(Rsdp),
        };

        let address = address.cast::<i8>();

        unsafe {
            let is_rsdp_valid = (0..size as usize)
                .map(|i| address.add(i))
                .fold(0, |acc, byte| acc + (*byte) as i32)
                & 0xff
                == 0;

            if !is_rsdp_valid {
                panic!("Failed to validated RSDP");
            }

            match rsdp.revision >= 2 && rsdp.xstd_address != 0 {
                true => {
                    let data = address.cast::<Xsdp>().read();
                    RootSystemDescriptorPointer::V2(data)
                }
                false => {
                    let data = address.cast::<Rsdp>().read();
                    RootSystemDescriptorPointer::V1(data)
                }
            }
        }
    }

    pub fn is_xsdt(&self) -> bool {
        if let RootSystemDescriptorPointer::V2(_) = self {
            true
        } else {
            false
        }
    }
}

static mut ROOT_SYSTEM_DESCRIPTOR_POINTER: OnceLock<RootSystemDescriptorPointer> = OnceLock::new();

pub fn initialize() {
    log::info!("Initializing RSDP");

    let rsdp_address = boot::RSDP_REQUEST
        .get_response()
        .get()
        .expect("Failed to get RSDP Response")
        .address
        .as_ptr()
        .unwrap();

    unsafe {
        ROOT_SYSTEM_DESCRIPTOR_POINTER.set(RootSystemDescriptorPointer::new(VirtualAddress::new(
            rsdp_address as u64,
        )))
    }
}
