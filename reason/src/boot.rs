use limine::{BaseRevision, FramebufferRequest, HhdmRequest, MemmapRequest};

static BASE_REVISION: BaseRevision = BaseRevision::new(1);
static HHDM_REQUEST : HhdmRequest = HhdmRequest::new(0);

pub static MEMORY_MAP_REQUEST: MemmapRequest = MemmapRequest::new(0);
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new(0);

pub static mut HHDM_OFFSET: u64 = 0;

pub fn initialize() {
    assert!(BASE_REVISION.is_supported());

    unsafe {
        HHDM_OFFSET = HHDM_REQUEST.get_response().get().expect("Failed to get hhdm request").offset;
    }
}
