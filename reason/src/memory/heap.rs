use crate::data_structures::{DoublyLinkedList, DoublyLinkedListNode};
use crate::memory::VirtualAddress;

pub struct ExplicitFreeList {
    total_allocated: u64,
    base: VirtualAddress,
}

impl ExplicitFreeList {
    pub fn new() -> Self {
        todo!()
    }
}
