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
use core::{
    cmp,
    ptr::{addr_of, NonNull},
};

use crate::memory::vmm::{VirtualMemoryManager, VirtualMemoryObject};
use crate::memory::VirtualAddress;
use crate::misc::utils::{align_up, size, OnceCellMutex};
use crate::{
    data_structures::{
        DoublyLinkedList, DoublyLinkedListNode, SinglyLinkedList, SinglyLinkedListNode,
    },
    misc::log,
};

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
    fn get_address(&self) -> VirtualAddress;

    unsafe fn get_header(&self) -> BlockHeader;

    unsafe fn get_size(&self) -> u16;
    unsafe fn get_payload_address(&self) -> VirtualAddress;
    unsafe fn get_is_used(&self) -> bool;

    unsafe fn install_headers(&self, size: u16);

    unsafe fn set_is_used(&self, value: bool);
}

impl BlockHeader {
    fn get_size(&self) -> u16 {
        self.data & !0b111_u16
    }

    fn get_is_used(&self) -> bool {
        self.data & 1 == 1
    }
}

impl BlockMetadata for NonNull<Block> {
    fn get_address(&self) -> VirtualAddress {
        VirtualAddress::new(self.as_ptr() as u64)
    }

    unsafe fn get_header(&self) -> BlockHeader {
        // ┌────────┬───────────┬─────────┐
        // │ Header │  Payload  │ Header  │
        // └────────┴───────────┴─────────┘
        // └─── self points here
        let addr = self.get_address().as_addr();
        (addr as *const BlockHeader).read()
    }

    unsafe fn get_size(&self) -> u16 {
        self.get_header().get_size()
    }

    unsafe fn get_payload_address(&self) -> VirtualAddress {
        // ┌────────┬───────────┬─────────┐
        // │ Header │  Payload  │ Header  │
        // └────────┴───────────┴─────────┘
        //          └─── We want this
        // └─── but `self` points here
        let addr = self.get_address().as_addr();
        (addr + size!(BlockHeader)).as_virtual()
    }

    unsafe fn get_is_used(&self) -> bool {
        self.get_header().get_is_used()
    }

    unsafe fn set_is_used(&self, set_use: bool) {
        let ptr = self.get_address().as_addr() as *mut BlockHeader;
        if set_use {
            (*ptr).data |= 1_u16
        } else {
            (*ptr).data &= !1_u16
        }
    }

    unsafe fn install_headers(&self, size: u16) {
        let addr = self.get_address().as_addr();

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

    unsafe fn iter_blocks(&self) -> HeapRegionIterator;

    unsafe fn get_current_address(&self) -> VirtualAddress;

    unsafe fn add_to_free_list(&mut self, block: NonNull<Block>);
    unsafe fn remove_from_free_list(&mut self, block: NonNull<Block>);
}

struct HeapRegionIterator {
    current: VirtualAddress,
    end: VirtualAddress,
}

impl Iterator for HeapRegionIterator {
    type Item = NonNull<Block>;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.current <= self.end);

        if self.current == self.end {
            return None;
        }

        let block = unsafe { NonNull::new_unchecked(self.current.as_addr() as *mut Block) };
        self.current += unsafe { block.get_size() as u64 } + 2 * size!(BlockHeader);

        Some(block)
    }
}

impl HeapRegionMetadata for NonNull<HeapRegion> {
    unsafe fn add_to_free_list(&mut self, block: NonNull<Block>) {
        self.as_mut()
            .free_blocks
            .append_to_address(block.get_payload_address(), Block {});

        block.set_is_used(false);
    }

    unsafe fn remove_from_free_list(&mut self, block: NonNull<Block>) {
        self.as_mut()
            .free_blocks
            .remove(|node| node.ptr_to_data() == block);
    }

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

        log::info!("length {}", object.length);

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

    unsafe fn iter_blocks(&self) -> HeapRegionIterator {
        HeapRegionIterator {
            current: self.get_payload_address(),
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
    pub regions: SinglyLinkedList<HeapRegion>,
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

        for node in self.regions.iter() {
            let region = &node.as_ref().data;

            for node in region.free_blocks.iter() {
                //          ┌─── block payload ────┐
                // ┌────────┬──────┬──────┬────────┬────────┐
                // │ header │ next │ prev │        │ header │
                // └────────┴──────┴──────┴────────┴────────┘
                // │        └─── node lives here
                // └─── we want to get this address to get the size

                let block_addr = addr_of!(node) as u64 - size!(BlockHeader);
                let block = NonNull::new_unchecked(block_addr as *mut Block);

                if block.get_is_used() && block.get_size() as u64 >= size {
                    block.set_is_used(true);

                    // TODO: take into account alignment stuff here
                    return block.get_payload_address();
                }
            }
        }

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

        self.regions.tail().unwrap_unchecked()
    }

    pub unsafe fn free(&mut self, address: VirtualAddress) {
        for node in self.regions.iter() {
            // ┌─ Heap Region Node ─┐
            // ┌──────┬─────────────┐
            // │ next │ Heap Region │
            // └──────┴─────────────┘
            // └─────── node points here

            let region_addr = addr_of!(node) as u64 + size!(Option<NonNull<()>>);
            let mut region = NonNull::new_unchecked(region_addr as *mut HeapRegion);

            // the address doesn't belong to this region
            if !(region.get_payload_address() >= address && address < region.get_end_address()) {
                log::info!("{}", region.get_payload_address());
                log::info!("{}", region.get_end_address());
                // log::info!("{}", address);

                continue;
            }

            for block in region.iter_blocks() {
                let begin = block.get_payload_address();
                let end = begin + block.get_size().into();

                // not within the block
                if !(begin >= address && address < end) {
                    continue;
                }

                // ┌────────── prev block  ───────┐                              ┌───────── next block  ───────┐
                // ┌────────┬───────────┬─────────┬────────┬───────────┬─────────┬────────┬───────────┬────────┐
                // │ Header │  Payload  │ Header  │ Header │  Payload  │ Header  │ Header │  Payload  │ Header │
                // └────────┴───────────┴─────────┴────────┴───────────┴─────────┴────────┴───────────┴────────┘
                //                                └────── block points here ─────┘
                let prev_block = 'prev_block: {
                    let block_addr = addr_of!(block) as u64;

                    let prev_header_addr = block_addr - size!(BlockHeader);
                    let prev_header = (prev_header_addr as *const BlockHeader).read();

                    let prev_block_addr =
                        block_addr - 2 * size!(BlockHeader) - prev_header.get_size() as u64;

                    if prev_header_addr < region.get_payload_address().as_addr() {
                        break 'prev_block None;
                    }

                    Some(NonNull::new_unchecked(prev_block_addr as *mut Block))
                };

                let next_block = 'next_block: {
                    let next_block_addr =
                        addr_of!(block) as u64 + 2 * size!(BlockHeader) + block.get_size() as u64;

                    if next_block_addr >= region.get_end_address().as_addr() {
                        break 'next_block None;
                    }

                    Some(NonNull::new_unchecked(next_block_addr as *mut Block))
                };

                match (prev_block, next_block) {
                    (None, None) => {
                        block.set_is_used(false);

                        region.add_to_free_list(block);
                    }
                    // ┌────────── prev block  ───────┐                              ┌───────── next block  ───────┐
                    // ┌────────┬───────────┬─────────┬────────┬───────────┬─────────┬────────┬───────────┬────────┐
                    // │ Header │  Payload  │ Header  │ Header │  Payload  │ Header  │ Header │  Payload  │ Header │
                    // └────────┴───────────┴─────────┴────────┴───────────┴─────────┴────────┴───────────┴────────┘
                    //                                └────── block points here ─────┘
                    //          └────────────────────────────── total size ───────────────────────────────┘
                    (Some(prev_block), Some(next_block)) => {
                        let total_size = prev_block.get_size()
                            + block.get_size()
                            + next_block.get_size()
                            + 4 * size!(BlockHeader) as u16;

                        region.remove_from_free_list(prev_block);
                        region.remove_from_free_list(next_block);
                        region.remove_from_free_list(block);

                        prev_block.install_headers(total_size);

                        region.add_to_free_list(prev_block);
                    }
                    // ┌────────── prev block  ───────┐                              ┌───────── next block  ───────┐
                    // ┌────────┬───────────┬─────────┬────────┬───────────┬─────────┬────────┬───────────┬────────┐
                    // │ Header │  Payload  │ Header  │ Header │  Payload  │ Header  │ Header │  Payload  │ Header │
                    // └────────┴───────────┴─────────┴────────┴───────────┴─────────┴────────┴───────────┴────────┘
                    //                                └────── block points here ─────┘
                    //          └────────── total size ────────────────────┘
                    (Some(prev_block), None) => {
                        let total_size = prev_block.get_size()
                            + block.get_size()
                            + 2 * size!(BlockHeader) as u16;

                        region.remove_from_free_list(prev_block);
                        region.remove_from_free_list(block);

                        prev_block.install_headers(total_size);

                        region.add_to_free_list(prev_block);
                    }

                    // ┌────────── prev block  ───────┐                              ┌───────── next block  ───────┐
                    // ┌────────┬───────────┬─────────┬────────┬───────────┬─────────┬────────┬───────────┬────────┐
                    // │ Header │  Payload  │ Header  │ Header │  Payload  │ Header  │ Header │  Payload  │ Header │
                    // └────────┴───────────┴─────────┴────────┴───────────┴─────────┴────────┴───────────┴────────┘
                    //                                └────── block points here ─────┘
                    //                                         └────────── total size ────────────────────┘
                    (None, Some(next_block)) => {
                        let total_size = block.get_size()
                            + next_block.get_size()
                            + 2 * size!(BlockHeader) as u16;

                        region.remove_from_free_list(block);
                        region.remove_from_free_list(next_block);

                        block.install_headers(total_size);

                        region.add_to_free_list(block);
                    }
                }
            }
        }
    }
}
