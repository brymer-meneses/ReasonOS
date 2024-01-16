mod bitmap_allocator;

pub mod pmm;
pub mod vmm;

pub fn initialize() {
    pmm::initialize();
}
