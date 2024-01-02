#![no_std]
#![no_main]

mod drivers;

use core::arch::asm;

use drivers::serial;

static FRAMEBUFFER_REQUEST: limine::FramebufferRequest = limine::FramebufferRequest::new(0);
static BASE_REVISION: limine::BaseRevision = limine::BaseRevision::new(1);

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    assert!(BASE_REVISION.is_supported());

    serial::initialize();

    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response().get() {
        if framebuffer_response.framebuffer_count < 1 {
            hcf();
        }
        let framebuffer = &framebuffer_response.framebuffers()[0];
        for y in 0..framebuffer.height as usize {
            for x in 0..framebuffer.width as usize {
                let pixel_offset =
                    y * framebuffer.pitch as usize + x * framebuffer.bpp as usize / 8;
                unsafe {
                    *(framebuffer.address.as_ptr().unwrap().add(pixel_offset) as *mut u32) =
                        0x11111B;
                }
            }
        }
    }

    serial::write_string("Hello kernel!");
    hcf();
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    hcf();
}

fn hcf() -> ! {
    unsafe {
        asm!("cli");
        loop {
            asm!("hlt");
        }
    }
}
