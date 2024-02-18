use limine::{BaseRevision, FramebufferRequest, HhdmRequest, KernelFileRequest, MemmapRequest};

use crate::memory::VirtualAddress;
use crate::misc::elf::{Elf64, SymbolType};
use crate::misc::log;

static BASE_REVISION: BaseRevision = BaseRevision::new(1);
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new(0);

pub static MEMORY_MAP_REQUEST: MemmapRequest = MemmapRequest::new(0);
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new(0);
pub static KERNEL_FILE_REQUEST: KernelFileRequest = KernelFileRequest::new(0);

pub static mut HHDM_OFFSET: u64 = 0;

pub fn initialize() {
    assert!(BASE_REVISION.is_supported());

    unsafe {
        HHDM_OFFSET = HHDM_REQUEST
            .get_response()
            .get()
            .expect("Failed to get hhdm request")
            .offset;
    }

    let kernel_file_address = {
        let response = KERNEL_FILE_REQUEST
            .get_response()
            .get()
            .expect("Failed to get Kernel Address");

        let file = response.kernel_file.get().unwrap();
        VirtualAddress::new(file.base.get().unwrap() as *const u8 as u64)
    };

    let kernel_elf = Elf64::new(kernel_file_address);

    for section in kernel_elf.section_headers() {
        log::info!(
            "{}",
            kernel_elf.resolve_symbol_name(section.name_offset as u64)
        );
    }

    // for symbol in kernel_elf.symbols() {
    //     log::info!("{}", symbol.name_offset);
    // }
}

#[allow(unused)]
pub fn get_symbol(address: VirtualAddress) {
    todo!()
}
