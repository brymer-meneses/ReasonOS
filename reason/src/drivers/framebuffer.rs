use crate::arch;

static FRAMEBUFFER_REQUEST: limine::FramebufferRequest = limine::FramebufferRequest::new(0);
static BASE_REVISION: limine::BaseRevision = limine::BaseRevision::new(1);

pub fn initialize() {
    assert!(BASE_REVISION.is_supported());

    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response().get() {
        if framebuffer_response.framebuffer_count < 1 {
            arch::hcf();
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
}