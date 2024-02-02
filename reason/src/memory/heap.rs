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
use core::cmp;
use core::ptr::NonNull;

use crate::data_structures::{
    DoublyLinkedList, DoublyLinkedListNode, SinglyLinkedList, SinglyLinkedListNode,
};
use crate::memory::vmm::{VirtualMemoryManager, VirtualMemoryObject};
use crate::memory::VirtualAddress;
use crate::misc::utils::{align_up, size, OnceCellMutex};

use super::IntoAddress;

/// A `Block` is deliberately zero-sized to make it so that
/// `NonNull<Block>` is effectively akin to a non null `*void` ptr in c
///
/// A block the following structure:
///
/// ┌────────┬───────────┬─────────┐
/// │ Header │  Payload  │ Header  │
/// └────────┴───────────┴─────────┘
/// └─── NonNull<Block> points here
#[repr(C)]
#[derive(Debug)]
struct Block;

#[repr(C)]
struct BlockHeader {
    data: u16,
}

trait BlockMetadata {
    unsafe fn get_header(&self) -> BlockHeader;
    unsafe fn get_size(&self) -> u16;
    unsafe fn get_payload_address(&self) -> VirtualAddress;
    unsafe fn get_is_used(&self) -> bool;

    unsafe fn install_headers(&self, size: u16);

    unsafe fn set_is_used(&self, value: bool);
}

impl BlockMetadata for NonNull<Block> {
    unsafe fn get_header(&self) -> BlockHeader {
        // ┌────────┬───────────┬─────────┐
        // │ Header │  Payload  │ Header  │
        // └────────┴───────────┴─────────┘
        // └─── self points here
        let addr = self.as_ptr() as u64;
        (addr as *const BlockHeader).read()
    }

    unsafe fn get_size(&self) -> u16 {
        self.get_header().data & !0b111_u16
    }

    unsafe fn get_payload_address(&self) -> VirtualAddress {
        // ┌────────┬───────────┬─────────┐
        // │ Header │  Payload  │ Header  │
        // └────────┴───────────┴─────────┘
        //          └─── We want this
        // └─── but `self` points here
        let addr = self.as_ptr() as u64;
        (addr + size!(BlockHeader)).as_virtual()
    }

    unsafe fn get_is_used(&self) -> bool {
        self.get_header().data & 1 == 1
    }

    unsafe fn set_is_used(&self, set_use: bool) {
        let ptr = self.as_ptr() as u64 as *mut BlockHeader;
        if set_use {
            (*ptr).data |= 1_u16
        } else {
            (*ptr).data &= !1_u16
        }
    }

    unsafe fn install_headers(&self, size: u16) {
        let addr = self.as_ptr() as u64;

        // ┌────────┬───────────┬─────────┐
        // │ Header │  Payload  │ Header  │
        // └────────┴───────────┴─────────┘
        // └─── `self` points here
        let start_header = addr as *mut BlockHeader;
        start_header.write(BlockHeader { data: size });

        let end_header = (addr + 2 * size!(BlockHeader) + size as u64) as *mut BlockHeader;
        end_header.write(BlockHeader { data: size });
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct HeapRegion {
    base: VirtualAddress,
    total_allocated: u64,
    free_blocks: DoublyLinkedList<Block>,
}

trait HeapRegionMetadata {
    unsafe fn get_object(&self) -> NonNull<VirtualMemoryObject>;

    unsafe fn get_capacity(&self) -> u64;
    unsafe fn allocate_block(&mut self, size: u16, alignment: u64) -> Option<VirtualAddress>;

    unsafe fn get_payload_address(&self) -> VirtualAddress;

    unsafe fn get_end_address(&self) -> VirtualAddress;
    unsafe fn get_total_allocated(&self) -> u64;

    unsafe fn get_blocks(&self) -> HeapRegionIterator;
    unsafe fn get_current_address(&self) -> VirtualAddress;
}

struct HeapRegionIterator {
    current: VirtualAddress,
    end: VirtualAddress,
}

impl HeapRegionMetadata for NonNull<HeapRegion> {
    unsafe fn get_object(&self) -> NonNull<VirtualMemoryObject> {
        let next_node_size = size!(Option<NonNull<SinglyLinkedListNode<HeapRegion>>>);

        let addr = self.as_ptr() as u64;
        let object_addr = addr - next_node_size - size!(VirtualMemoryObject);

        assert_ne!(object_addr, 0);
        NonNull::new_unchecked(object_addr as *mut VirtualMemoryObject)
    }

    unsafe fn get_total_allocated(&self) -> u64 {
        self.as_ref().total_allocated
    }

    unsafe fn get_capacity(&self) -> u64 {
        let object = self.get_object().as_ref();
        let total_heap_region_size = size!(HeapRegion) + size!(Option<NonNull<()>>);
        let total_vm_object_size = size!(VirtualMemoryObject) + size!(Option<NonNull<()>>);

        let length = object.length - (total_heap_region_size + total_vm_object_size);

        length
    }

    unsafe fn get_end_address(&self) -> VirtualAddress {
        let end_address = self.as_ref().base + self.get_capacity();
        assert!(end_address.is_page_aligned());

        end_address
    }

    unsafe fn get_payload_address(&self) -> VirtualAddress {
        // ┌─ Heap Region Node ─┐
        // ┌──────┬─────────────┬────────────┐
        // │ next │ Heap Region │ payload    │
        // │      │   Fields    │            │
        // └──────┴─────────────┴────────────┘
        //                      └────────────── We want this
        //        └────────────── NonNull<HeapRegion> points here
        let addr = self.as_ptr() as u64;
        let payload_addr = addr + size!(HeapRegion);
        payload_addr.as_virtual()
    }

    unsafe fn get_current_address(&self) -> VirtualAddress {
        self.as_ref().base + self.as_ref().total_allocated
    }

    unsafe fn get_blocks(&self) -> HeapRegionIterator {
        HeapRegionIterator {
            current: self.get_current_address(),
            end: self.get_end_address(),
        }
    }

    unsafe fn allocate_block(&mut self, size: u16, alignment: u64) -> Option<VirtualAddress> {
        // ensure that we can store memory here
        // debug_assert!(size >= size!(DoublyLinkedListNode<Block>) as u16);

        let current_address = self.get_current_address();

        let (total_size, padding) = {
            let total_allocated = self.get_total_allocated();

            let payload_address = current_address + size!(BlockHeader);
            let aligned_address = align_up(payload_address.as_addr(), alignment).as_virtual();

            let padding = aligned_address.as_addr() - payload_address.as_addr();

            let aligned_size = size + padding as u16;

            let total_size = cmp::max(
                align_up(aligned_size as u64, 8),
                size!(DoublyLinkedListNode<Block>),
            );

            if total_size + total_allocated > self.get_capacity() {
                return None;
            }

            self.as_mut().total_allocated += total_size;
            (total_size, padding)
        };

        debug_assert_eq!(total_size % 8, 0);
        debug_assert_eq!(current_address.as_addr() % 2, 0);

        let block = {
            let block = NonNull::new_unchecked(current_address.as_addr() as *mut Block);
            block.install_headers(total_size as u16);
            block.set_is_used(true);
            block
        };

        Some(block.get_payload_address() + padding)
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

    pub unsafe fn alloc(&mut self, size: u64, alignment: u64) -> VirtualAddress {
        assert!(size != 0);
        assert!(size < u16::MAX.into());

        let mut current_region = match self.regions.tail() {
            None => self.allocate_region(size),
            Some(region) => region,
        };

        match current_region.allocate_block(size as u16, alignment) {
            None => {
                self.allocate_region(size)
                    // we can safely unwrap here since self.allocate_region
                    // allocates a page aligned size which is definitely bigger than size
                    .allocate_block(size as u16, alignment)
                    .unwrap_unchecked()
            }
            Some(addr) => addr,
        }
    }

    pub unsafe fn free(&mut self, address: VirtualAddress) {
        if address.is_page_aligned() {
            self.vmm.as_mut().lock().free_object(address);
        }
    }

    unsafe fn allocate_region(&mut self, size: u64) -> NonNull<HeapRegion> {
        let mut vmm = self.vmm.as_mut().lock();
        let object = vmm.allocate_object(size);

        let object_base = object.as_ref().base;
        let heap_region_base = object_base + self.regions.list_node_size();
        self.regions.append_to_address(
            object_base,
            HeapRegion {
                base: heap_region_base,
                free_blocks: DoublyLinkedList::new(),
                total_allocated: 0,
            },
        );

        self.regions.tail().unwrap()
    }
}
