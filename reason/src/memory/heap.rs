use core::cmp;

use crate::arch::paging::PAGE_SIZE;
use crate::data_structures::{
    DoublyLinkedList, DoublyLinkedListNode, SinglyLinkedList, SinglyLinkedListNode,
};
use crate::memory::vmm::{VirtualMemoryManager, VirtualMemoryObject};
use crate::memory::VirtualAddress;
use crate::misc::utils::{align_up, size, OnceCellMutex};

const NODE_SIZE: u64 = size!(SinglyLinkedListNode<VirtualMemoryObject>);

/// A block has the following layout
/// +--------+---------+--------+
/// | Header | Payload | Header |
/// +--------+---------+--------+
struct Block;

#[derive(Clone, Copy)]
struct Header {
    data: u16,
}

pub struct ExplicitFreeList {
    total_allocated_to_current_object: u64,
    free_blocks: DoublyLinkedList<Block>,
    objects: SinglyLinkedList<VirtualMemoryObject>,
    vmm: *mut OnceCellMutex<VirtualMemoryManager>,
}

impl ExplicitFreeList {
    pub fn new(vmm: *mut OnceCellMutex<VirtualMemoryManager>) -> Self {
        Self {
            total_allocated_to_current_object: 0,
            vmm,
            free_blocks: DoublyLinkedList::new(),
            objects: SinglyLinkedList::new(),
        }
    }

    /// internal allocation function, because we keep track our own singly linked list
    /// for allocated vm objects separate from the `vmm`. This function basically calls
    /// `vmm.allocate_object` and adjusts the base to account for the node size
    unsafe fn allocate_object(&mut self, size: u64) -> VirtualMemoryObject {
        let size = align_up(size + NODE_SIZE, PAGE_SIZE);
        match (*self.vmm).lock().allocate_object(size) {
            None => panic!("Failed to allocate an object"),
            Some(mut object) => {
                let allocate_to = object.base;

                // offset this because we keep track of our own allocated vm objects
                // and we store the node to `object.base`
                object.base += NODE_SIZE;
                self.objects.append(*object, allocate_to);
                return *object;
            }
        }
    }

    /// gets a copy of the current object that can fit the give size
    /// if not then it initializes a new object
    unsafe fn ensure_object_capacity(&mut self, size: u64) -> VirtualMemoryObject {
        match self.objects.tail() {
            None => self.allocate_object(size),
            Some(object) => {
                if object.length <= size + self.total_allocated_to_current_object {
                    self.total_allocated_to_current_object = 0;
                    return self.allocate_object(size);
                }
                *object
            }
        }
    }

    /// # params:
    /// `size` - size of allocations in bytes
    pub unsafe fn alloc(&mut self, size: u64) -> VirtualAddress {
        // TODO: check first if we can find a suitable object or region before allocating

        if size > PAGE_SIZE {
            return self.allocate_object(size).base;
        }

        let size = cmp::max(size, size!(DoublyLinkedListNode<*mut Block>));

        let current_object = self.ensure_object_capacity(size);
        let block = Block::new_from_address(
            current_object.base + self.total_allocated_to_current_object,
            // we can be sure that `size` is less than 4096 because we allocate a vm object if the
            // size is greater than or equal to `PAGE_SIZE`
            size as u16,
        );
        self.total_allocated_to_current_object += size;

        (*block).payload()
    }
}

impl Block {
    /// Instantiates a new block ptr from `base`
    unsafe fn new_from_address(base: VirtualAddress, size: u16) -> *const Block {
        let size = (size as u64 + size!(DoublyLinkedListNode<*const Block>)) as u16;

        let address = base.as_addr();
        {
            let left_header = address as *mut Header;
            left_header.write(Header { data: size });

            let right_header = (address + size!(Header) + size as u64) as *mut Header;
            right_header.write(Header { data: size });
        }

        address as *const Block
    }

    fn base(&self) -> VirtualAddress {
        let addr = self as *const _ as u64;
        VirtualAddress::new(addr)
    }

    fn header(&self) -> Header {
        let base = self.base();
        let header = base.as_addr() as *const Header;

        // mask the three bits since these bits are used to encode metadata
        unsafe { *header }
    }

    fn size(&self) -> u16 {
        self.header().size()
    }

    fn is_used(&self) -> bool {
        self.header().is_used()
    }

    fn payload(&self) -> VirtualAddress {
        let addr = self as *const _ as u64;
        VirtualAddress::new(addr + size!(Header))
    }

    unsafe fn next(&self) -> *mut Block {
        let next_address = self.base().as_addr() + 2 * size!(Header) + self.size() as u64;
        next_address as *mut Block
    }

    unsafe fn prev(&self) -> *mut Block {
        let previous_block_header = (self.base().as_addr() - size!(Header)) as *mut Header;
        let previous_block_size = (*previous_block_header).size() as u64;

        let block_address = self.base().as_addr() - 2 * size!(Header) - previous_block_size;

        block_address as *mut Block
    }
}

impl Header {
    fn size(&self) -> u16 {
        self.data & !0b111_u16
    }

    fn is_used(&self) -> bool {
        self.data & 1 == 1
    }
}
