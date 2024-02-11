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
use crate::memory::IntoAddress;
use crate::misc::log;
use crate::misc::utils::{align_up, size, OnceCellMutex};
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
    unsafe fn get_payload_size(&self) -> u32;
    unsafe fn install_headers(&self, header: BlockHeader);
    unsafe fn get_payload_address(&self) -> VirtualAddress;
    unsafe fn get_address(&self) -> VirtualAddress;

    unsafe fn set_is_used(&mut self, value: bool);
    unsafe fn get_is_used(&self) -> bool;
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

    unsafe fn get_payload_size(&self) -> u32 {
        self.get_header().size
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

    unsafe fn set_is_used(&mut self, value: bool) {
        let address = self.get_address().as_addr();
        let left_header = address as *mut BlockHeader;

        left_header.as_mut().unwrap().is_used = value;

        let right_header =
            (address + size!(BlockHeader) + left_header.as_ref().unwrap().size as u64)
                as *mut BlockHeader;

        right_header.as_mut().unwrap().is_used = value;
    }

    unsafe fn get_is_used(&self) -> bool {
        self.get_header().is_used
    }
}

struct HeapRegion {
    total_allocated: u64,
    free_blocks: DoublyLinkedList<Block>,
}

struct HeapRegionIterator {
    current_address: VirtualAddress,
    end_address: VirtualAddress,
}

trait HeapRegionPtr {
    unsafe fn allocate_block(&mut self, size: u64) -> Option<NonNull<Block>>;
    unsafe fn allocate_block_aligned(
        &mut self,
        size: u64,
        alignment: u64,
    ) -> Option<(NonNull<Block>, u64)>;

    unsafe fn get_virtual_memory_object(&self) -> NonNull<VirtualMemoryObject>;

    unsafe fn get_end_address(&self) -> VirtualAddress;

    unsafe fn get_current_address(&self) -> VirtualAddress;
    unsafe fn get_payload_address(&self) -> VirtualAddress;
    unsafe fn get_address(&self) -> VirtualAddress;

    unsafe fn iter_blocks(&self) -> HeapRegionIterator;

    unsafe fn add_to_free_list(&mut self, block: NonNull<Block>);
    unsafe fn remove_from_free_list(&mut self, block: NonNull<Block>);
}

impl HeapRegionPtr for NonNull<HeapRegion> {
    unsafe fn allocate_block(&mut self, size: u64) -> Option<NonNull<Block>> {
        debug_assert!(size < u32::MAX.into());

        let current_address = self.get_current_address();

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
            is_used: true,
        });

        Some(block)
    }

    unsafe fn allocate_block_aligned(
        &mut self,
        size: u64,
        alignment: u64,
    ) -> Option<(NonNull<Block>, u64)> {
        debug_assert!(size < u32::MAX.into());

        let current_address = self.get_current_address();

        debug_assert!(
            current_address.is_aligned_to(8),
            "address to put a block must be 8 aligned!"
        );

        let padding = {
            let payload_address = (current_address + size!(BlockHeader)).as_addr();
            let aligned_address = align_up(payload_address, alignment);
            aligned_address - payload_address
        };

        let size = cmp::max(size, size!(DoublyLinkedListNode<Block>));
        let total_size = 2 * size!(BlockHeader) + size;

        if current_address + total_size > self.get_end_address() {
            return None;
        }

        self.as_mut().total_allocated += total_size;

        let block = NonNull::new_unchecked(current_address.as_addr() as *mut Block);

        block.install_headers(BlockHeader {
            size: size as u32,
            is_used: true,
        });

        Some((block, padding))
    }

    unsafe fn add_to_free_list(&mut self, mut block: NonNull<Block>) {
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

    /// gets the address where we can place a new block this will be equal to the output of `get_end_address` if the region is filled
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

    unsafe fn iter_blocks(&self) -> HeapRegionIterator {
        HeapRegionIterator {
            current_address: self.get_payload_address(),
            end_address: self.get_current_address(),
        }
    }
}

impl Iterator for HeapRegionIterator {
    type Item = NonNull<Block>;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.current_address <= self.end_address);

        if self.current_address == self.end_address {
            return None;
        }

        let block = unsafe { NonNull::new_unchecked(self.current_address.as_addr() as *mut Block) };
        let payload_size = unsafe { block.get_header().size } as u64;

        self.current_address += payload_size + 2 * size!(BlockHeader);

        Some(block)
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

    pub unsafe fn alloc_aligned(&mut self, size: u64, alignment: u64) -> VirtualAddress {
        for region in self
            .regions
            .iter_nodes()
            .map(|mut node| node.as_mut().ptr_to_data().read())
        {
            for node in region.free_blocks.iter_nodes() {
                //          ┌─── block payload ────┐
                // ┌────────┬──────┬──────┬────────┬────────┐
                // │ header │ next │ prev │        │ header │
                // └────────┴──────┴──────┴────────┴────────┘
                // │        └─── node lives here
                // └─── we want to get this address to get the size

                let block_addr = node.as_ptr() as u64 - size!(BlockHeader);
                let mut block = NonNull::new_unchecked(block_addr as *mut Block);

                if block.get_is_used() {
                    continue;
                }

                let aligned_address = align_up(block.get_payload_address().as_addr(), alignment);
                let padding = aligned_address - block.get_payload_size() as u64;

                if block.get_payload_size() as u64 >= size + padding {
                    block.set_is_used(true);
                    return block.get_payload_address() + padding;
                }
            }
        }

        let mut current_region = match self.regions.tail() {
            None => self.allocate_region(size),
            Some(region) => region,
        };

        let (block, padding) = match current_region.allocate_block_aligned(size, alignment) {
            None => self
                .allocate_region(size)
                .allocate_block_aligned(size, alignment)
                .unwrap_unchecked(),

            Some((block, padding)) => (block, padding),
        };

        block.get_payload_address() + padding
    }

    pub unsafe fn alloc(&mut self, size: u64) -> VirtualAddress {
        for mut region in self
            .regions
            .iter_nodes()
            .map(|mut node| node.as_mut().ptr_to_data())
        {
            for node in region.as_ref().free_blocks.iter_nodes() {
                //          ┌─── block payload ────┐
                // ┌────────┬──────┬──────┬────────┬────────┐
                // │ header │ next │ prev │        │ header │
                // └────────┴──────┴──────┴────────┴────────┘
                // │        └─── node lives here
                // └─── we want to get this address to get the size

                let block_addr = node.as_ptr() as u64 - size!(BlockHeader);
                let mut block = NonNull::new_unchecked(block_addr as *mut Block);

                if !block.get_is_used() && block.get_payload_size() as u64 >= size {
                    block.set_is_used(true);
                    region.remove_from_free_list(block);

                    return block.get_payload_address();
                };
            }
        }

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

    pub unsafe fn free(&mut self, address: VirtualAddress) {
        for mut region in self
            .regions
            .iter_nodes()
            .map(|mut n| n.as_mut().ptr_to_data())
        {
            if !(region.get_payload_address() <= address && address < region.get_end_address()) {
                continue;
            }

            for mut block in region.iter_blocks() {
                let block_size = block.get_header().size as u64;

                let payload_start_address = block.get_payload_address();
                let payload_end_address = payload_start_address + block_size;

                let block_start_address = block.as_ptr() as u64;
                let block_end_address = block_start_address + block_size + 2 * size!(BlockHeader);

                if !(payload_start_address <= address && address < payload_end_address) {
                    continue;
                }

                // ┌────────── prev block  ───────┐                              ┌───────── next block  ───────┐
                // ┌────────┬───────────┬─────────┬────────┬───────────┬─────────┬────────┬───────────┬────────┐
                // │ Header │  Payload  │ Header  │ Header │  Payload  │ Header  │ Header │  Payload  │ Header │
                // └────────┴───────────┴─────────┴────────┴───────────┴─────────┴────────┴───────────┴────────┘
                //                                └────── block_address

                let prev_block = 'prev_block: {
                    let header_address = block_start_address - size!(BlockHeader);

                    if header_address < region.get_payload_address().as_addr() {
                        break 'prev_block None;
                    }

                    let header = (header_address as *const BlockHeader).read();

                    let prev_block_address =
                        block_start_address - header.size as u64 - 2 * size!(BlockHeader);
                    let prev_block = NonNull::new_unchecked(prev_block_address as *mut Block);

                    if prev_block.get_is_used() {
                        break 'prev_block None;
                    }

                    Some(prev_block)
                };

                let next_block = 'next_block: {
                    let header_address = block_end_address + size!(BlockHeader);

                    if header_address.as_virtual() >= region.get_current_address() {
                        break 'next_block None;
                    }

                    let next_block_address = block_end_address;
                    let next_block = NonNull::new_unchecked(next_block_address as *mut Block);

                    if next_block.get_is_used() {
                        break 'next_block None;
                    }

                    Some(next_block)
                };

                match (prev_block, next_block) {
                    (None, None) => {
                        region.add_to_free_list(block);
                        block.set_is_used(false);
                    }

                    // ┌────────── prev block  ───────┐                              ┌───────── next block  ───────┐
                    // ┌────────┬───────────┬─────────┬────────┬───────────┬─────────┬────────┬───────────┬────────┐
                    // │ Header │  Payload  │ Header  │ Header │  Payload  │ Header  │ Header │  Payload  │ Header │
                    // └────────┴───────────┴─────────┴────────┴───────────┴─────────┴────────┴───────────┴────────┘
                    //                                └────── block points here ─────┘
                    //          └────────────────────────────── total size ───────────────────────────────┘
                    (Some(prev_block), Some(next_block)) => {
                        let total_size = prev_block.get_payload_size()
                            + block.get_payload_size()
                            + next_block.get_payload_size()
                            + 4 * size!(BlockHeader) as u32;

                        region.remove_from_free_list(prev_block);
                        region.remove_from_free_list(next_block);

                        prev_block.install_headers(BlockHeader {
                            size: total_size,
                            is_used: false,
                        });

                        region.add_to_free_list(prev_block);
                    }
                    // ┌────────── prev block  ───────┐                              ┌───────── next block  ───────┐
                    // ┌────────┬───────────┬─────────┬────────┬───────────┬─────────┬────────┬───────────┬────────┐
                    // │ Header │  Payload  │ Header  │ Header │  Payload  │ Header  │ Header │  Payload  │ Header │
                    // └────────┴───────────┴─────────┴────────┴───────────┴─────────┴────────┴───────────┴────────┘
                    //                                └────── block points here ─────┘
                    //          └────────── total size ────────────────────┘
                    (Some(prev_block), None) => {
                        let total_size = prev_block.get_payload_size()
                            + block.get_payload_size()
                            + 2 * size!(BlockHeader) as u32;

                        region.remove_from_free_list(prev_block);

                        prev_block.install_headers(BlockHeader {
                            size: total_size,
                            is_used: false,
                        });

                        region.add_to_free_list(prev_block);
                    }

                    // ┌────────── prev block  ───────┐                              ┌───────── next block  ───────┐
                    // ┌────────┬───────────┬─────────┬────────┬───────────┬─────────┬────────┬───────────┬────────┐
                    // │ Header │  Payload  │ Header  │ Header │  Payload  │ Header  │ Header │  Payload  │ Header │
                    // └────────┴───────────┴─────────┴────────┴───────────┴─────────┴────────┴───────────┴────────┘
                    //                                └────── block points here ─────┘
                    //                                         └────────── total size ────────────────────┘
                    (None, Some(next_block)) => {
                        let total_size = block.get_payload_size()
                            + next_block.get_payload_size()
                            + 2 * size!(BlockHeader) as u32;

                        region.remove_from_free_list(next_block);

                        block.install_headers(BlockHeader {
                            size: total_size,
                            is_used: false,
                        });

                        region.add_to_free_list(block);
                    }
                }
            }
        }
    }
}
