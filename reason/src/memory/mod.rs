mod bitmap_allocator;
pub mod pmm;

pub fn initialize() {
    pmm::initialize();
}

