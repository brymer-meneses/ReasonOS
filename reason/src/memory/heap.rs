/// Visualizations
///
///            ┌─────── Heap Region ─────────┐
/// ┌──────────┬──────────┬──────────────────┐
/// │ Virtual  │ Heap     │                  │
/// │ Memory   │ Region   │     Payload      │
/// │ Object   │ Node     │                  │
/// │ Node     │          │                  │
/// └──────────┴──────────┴──────────────────┘
/// └───────── Virtual Memory Object  ───────┘
///
/// A heap region lives inside the `free_blocks` field
/// of the `ExplicitFreeList.`
///
/// ┌─ Virtual Memory Object Node ─┐
/// ┌──────┬───────────────────────┐
/// │ next │ Virtual Memory Object │
/// │      │      Fields           │
/// └──────┴───────────────────────┘
///        └───── NonNull<VirtualMemoryObject> points here
///
/// ┌─ Heap Region Node ─┐
/// ┌──────┬─────────────┐
/// │ next │ Heap Region │
/// │      │   Fields    │
/// └──────┴─────────────┘
///        └────────────── NonNull<HeapRegion> points here
///
/// The heap payload consists of these `Block`
///
/// ┌────────┬───────────┬─────────┐
/// │ Header │  Payload  │ Header  │
/// └────────┴───────────┴─────────┘
/// └─── NonNull<Block> points here
///
/// A `Block` is zero sized to make it so that `NonNull<Block>` is an opaque type
///
/// ┌────────┬───────────┬─────────┐
/// │ Header │  Payload  │ Header  │
/// └────────┴───────────┴─────────┘
/// └─── NonNull<Block> points here
///
use crate::data_structures::{DoublyLinkedList, DoublyLinkedListNode, SinglyLinkedList};
use crate::misc::log;
use crate::misc::utils::{size, OnceCellMutex};
use core::cmp;
use core::ptr::NonNull;

use super::vmm::{VirtualMemoryManager, VirtualMemoryObject};
use super::VirtualAddress;

const NEXT_PTR_SIZE: u64 = size!(Option<NonNull<()>>);

struct Block;

#[repr(align(8))]
#[derive(Clone, Copy)]
struct BlockHeader {
    size: u32,
    is_used: bool,
}

trait BlockPtr {
    unsafe fn get_header(&self) -> BlockHeader;
    unsafe fn install_headers(&self, header: BlockHeader);
    unsafe fn get_payload_address(&self) -> VirtualAddress;
    unsafe fn get_address(&self) -> VirtualAddress;
}

impl BlockPtr for NonNull<Block> {
    unsafe fn get_address(&self) -> VirtualAddress {
        VirtualAddress::new(self.as_ptr() as u64)
    }

    unsafe fn get_header(&self) -> BlockHeader {
        let header = self.get_address().as_addr() as *mut BlockHeader;
        header.read()
    }

    unsafe fn get_payload_address(&self) -> VirtualAddress {
        self.get_address() + size!(BlockHeader)
    }

    unsafe fn install_headers(&self, header: BlockHeader) {
        //
        // ┌────────┬───────────┬─────────┐
        // │ Header │  Payload  │ Header  │
        // └────────┴───────────┴─────────┘
        // └─── NonNull<Block> points here
        let address = self.get_address().as_addr();
        let left_header = address as *mut BlockHeader;
        left_header.write(header);

        let right_header = (address + size!(BlockHeader) + header.size as u64) as *mut BlockHeader;
        right_header.write(header);
    }
}

struct HeapRegion {
    total_allocated: u64,
    free_blocks: DoublyLinkedList<Block>,
}

trait HeapRegionPtr {
    unsafe fn allocate_block(&mut self, size: u64) -> Option<NonNull<Block>>;

    unsafe fn get_virtual_memory_object(&self) -> NonNull<VirtualMemoryObject>;

    unsafe fn get_end_address(&self) -> VirtualAddress;

    unsafe fn get_current_address(&self) -> VirtualAddress;
    unsafe fn get_payload_address(&self) -> VirtualAddress;
    unsafe fn get_address(&self) -> VirtualAddress;
}

impl HeapRegionPtr for NonNull<HeapRegion> {
    unsafe fn allocate_block(&mut self, size: u64) -> Option<NonNull<Block>> {
        debug_assert!(size < u32::MAX.into());

        let current_address = self.get_current_address();
        log::info!("current_address {current_address}");

        debug_assert!(
            current_address.is_aligned_to(8),
            "address to put a block must be 8 aligned!"
        );

        let size = cmp::max(size, size!(DoublyLinkedListNode<Block>));
        let total_size = 2 * size!(BlockHeader) + size;

        if current_address + total_size > self.get_end_address() {
            return None;
        }

        self.as_mut().total_allocated += total_size;

        let block = NonNull::new_unchecked(current_address.as_addr() as *mut Block);

        block.install_headers(BlockHeader {
            size: size as u32,
            is_used: false,
        });

        Some(block)
    }

    /// gets the virtual memory object that is located behind this ptr
    unsafe fn get_virtual_memory_object(&self) -> NonNull<VirtualMemoryObject> {
        //                   Heap Region Node
        //                  ┌────────────────┐
        // ┌──────┬─────────┬──────┬─────────┬───────────────────┐
        // │ next │ Virtual │ next │ Heap    │ Payload           │
        // │      │ Memory  │      │ Region  │                   │
        // │      │ Object  │      │ Fields  │                   │
        // │      │ Fields  │      │         │                   │
        // └──────┴─────────┴──────┴─────────┴───────────────────┘
        //        │                └────── NonNull<HeapRegion> points here
        //        └────── we want to get this

        let address = self.get_address().as_addr() - NEXT_PTR_SIZE - size!(VirtualMemoryObject);

        NonNull::new_unchecked(address as *mut VirtualMemoryObject)
    }

    unsafe fn get_end_address(&self) -> VirtualAddress {
        let object = self.get_virtual_memory_object();
        let beginning_address = object.as_ptr() as u64 - NEXT_PTR_SIZE;

        VirtualAddress::new(beginning_address + object.as_ref().length)
    }

    /// gets the address where we can place a new block
    unsafe fn get_current_address(&self) -> VirtualAddress {
        unsafe { self.get_payload_address() + self.as_ref().total_allocated }
    }

    /// gets the start where the blocks are placed
    unsafe fn get_payload_address(&self) -> VirtualAddress {
        self.get_address() + size!(HeapRegion)
    }

    /// gets the address of this ptr
    unsafe fn get_address(&self) -> VirtualAddress {
        VirtualAddress::new(self.as_ptr() as u64)
    }
}

pub struct ExplicitFreeList {
    vmm: NonNull<OnceCellMutex<VirtualMemoryManager>>,
    regions: SinglyLinkedList<HeapRegion>,
}

impl ExplicitFreeList {
    pub fn new(vmm: NonNull<OnceCellMutex<VirtualMemoryManager>>) -> Self {
        Self {
            vmm,
            regions: SinglyLinkedList::new(),
        }
    }

    pub unsafe fn alloc(&mut self, size: u64) -> VirtualAddress {
        let mut current_region = match self.regions.tail() {
            None => self.allocate_region(size),
            Some(region) => region,
        };

        let block = match current_region.allocate_block(size) {
            None => self
                .allocate_region(size)
                .allocate_block(size)
                .unwrap_unchecked(),

            Some(block) => block,
        };

        block.get_payload_address()
    }

    pub unsafe fn free(&mut self, _address: VirtualAddress) {
        // for region in self.regions.iter() {}
        todo!()
    }

    unsafe fn allocate_region(&mut self, size: u64) -> NonNull<HeapRegion> {
        let object = self.vmm.as_mut().lock().allocate_object(size).as_mut();

        log::info!("heap region allocated at {}", object.base);
        self.regions.append_to_address(
            object.base,
            HeapRegion {
                total_allocated: 0,
                free_blocks: DoublyLinkedList::new(),
            },
        );

        self.regions.tail().unwrap_unchecked()
    }
}
