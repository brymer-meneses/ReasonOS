use crate::arch::cpu;

use crate::boot::FRAMEBUFFER_REQUEST;

pub fn initialize() {
    let framebuffer_response = FRAMEBUFFER_REQUEST
        .get_response()
        .get()
        .expect("Failed to get framebuffer request");
    if framebuffer_response.framebuffer_count < 1 {
        cpu::hcf();
    }

    let framebuffer = &framebuffer_response.framebuffers()[0];
    for y in 0..framebuffer.height as usize {
        for x in 0..framebuffer.width as usize {
            let pixel_offset = y * framebuffer.pitch as usize + x * framebuffer.bpp as usize / 8;
            unsafe {
                *(framebuffer.address.as_ptr().unwrap().add(pixel_offset) as *mut u32) = 0x11111B;
            }
        }
    }
}
