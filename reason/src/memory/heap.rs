use core::cmp;
use core::ptr::addr_of;
use core::ptr::NonNull;

use crate::arch::paging::PAGE_SIZE;
use crate::data_structures::{
    DoublyLinkedList, DoublyLinkedListNode, SinglyLinkedList, SinglyLinkedListNode,
};
use crate::memory::vmm::{VirtualMemoryManager, VirtualMemoryRegion};
use crate::memory::VirtualAddress;
use crate::misc::log;
use crate::misc::utils::{align_up, size, OnceCellMutex};

use super::IntoAddress;

/// A `BlockMetadata` is a "view" of a `Block`
struct BlockMetadata {
    header: BlockHeader,

    /// start of the memory we can use
    base: VirtualAddress,
}

#[derive(Clone, Copy)]
struct BlockHeader {
    data: u16,
}

/// A `HeapRegion` is where we store the metadata of a `VirtualMemoryRegion`
/// we could technically just add the required attributes to `VirtualMemoryRegion`
/// but that will only be used by the `Heap` and is not ideal in my opinion
///
///  +-----------+-------------+-------+-------+-------+-------+
///  | VM Region | Heap Region | Block | Block | Block | Block |
///  +-----------+-------------+-------+-------+-------+-------+
///
pub struct HeapRegion {
    region: NonNull<VirtualMemoryRegion>,
    free_blocks: DoublyLinkedList<NonNull<BlockHeader>>,
    total_allocated: u16,
}

pub struct ExplicitFreeList {
    vmm: *mut OnceCellMutex<VirtualMemoryManager>,
    regions: SinglyLinkedList<HeapRegion>,
}

struct HeapRegionIterator {
    current: VirtualAddress,
    end: VirtualAddress,
}

impl BlockHeader {
    fn length(&self) -> u16 {
        self.data & !0b111_u16
    }

    fn is_used(&self) -> bool {
        self.data & 1 == 1
    }

    fn get_metadata(&self) -> BlockMetadata {
        let base = self as *const _ as u64;
        BlockMetadata {
            header: *self,
            base: base.as_virtual(),
        }
    }
}

impl Iterator for HeapRegionIterator {
    type Item = BlockMetadata;
    fn next(&mut self) -> Option<Self::Item> {
        assert!(self.current <= self.end);

        if self.current == self.end {
            return None;
        }

        let header = unsafe { (self.current.as_addr() as *const BlockHeader).read() };
        let base = self.current + header.length().into();

        self.current += header.length().into();

        Some(BlockMetadata { header, base })
    }
}

impl ExplicitFreeList {
    pub fn new(vmm: *mut OnceCellMutex<VirtualMemoryManager>) -> Self {
        Self {
            vmm,
            regions: SinglyLinkedList::new(),
        }
    }

    unsafe fn allocate_heap_region(&mut self, size: u64) -> *mut HeapRegion {
        let vm_region = (*self.vmm).lock().allocate_region(size);
        match vm_region {
            None => panic!("Failed to allocate an object"),
            Some(region) => {
                self.regions.append(
                    (*region).base,
                    HeapRegion {
                        total_allocated: 0,
                        free_blocks: DoublyLinkedList::new(),
                        region: NonNull::new_unchecked(region),
                    },
                );

                (*region).base += self.regions.list_node_size();
                return self.regions.tail_mut().unwrap();
            }
        }
    }

    unsafe fn get_appropriate_heap_region(&mut self, size: u64) -> *mut HeapRegion {
        match self.regions.tail_mut() {
            None => self.allocate_heap_region(size),
            Some(region_ptr) => {
                let region = unsafe { region_ptr.read() };
                log::info!("size {size}");
                log::info!("total_allocated {}", region.total_allocated);
                if region.length() <= size + region.total_allocated as u64 {
                    return self.allocate_heap_region(size);
                }
                region_ptr
            }
        }
    }
    pub unsafe fn alloc(&mut self, size: u64) -> VirtualAddress {
        // for mut node in self.regions.iter() {
        //     for block_metadata in node.read().data.blocks() {
        //         assert_ne!(block_metadata.header.data, 0);
        //     }
        // }

        // let the vmm manage this allocation since it is too big
        if size >= PAGE_SIZE {
            return self.allocate_heap_region(size).read().get_base_address();
        }

        // we adjust the size since we want the ability to store two pointers in a block
        let adjusted_size = cmp::max(size, size!(DoublyLinkedListNode<NonNull<BlockHeader>>));

        let region = self.get_appropriate_heap_region(adjusted_size);

        assert!(!region.is_null());

        let block_metadata = (*region).try_allocate_block(adjusted_size as u16);

        match block_metadata {
            Some(data) => data.base,
            None => panic!("unreachable!"),
        }
    }
}

impl HeapRegion {
    fn blocks(&self) -> HeapRegionIterator {
        // region.base points to the start of the heap region, so to get the address of the first
        // block we need to add the size of `HeapRegion`
        let region = unsafe { self.region.read() };
        let base = region.base + size!(HeapRegion);

        // we end at the "total_allocated"
        let end = base + self.total_allocated.into();

        HeapRegionIterator { current: base, end }
    }

    fn length(&self) -> u64 {
        let region = unsafe { self.region.as_ref() };

        region.length - size!(SinglyLinkedListNode<HeapRegion>)
    }

    fn get_base_address(&self) -> VirtualAddress {
        unsafe { self.region.read().base }
    }

    fn try_allocate_block(&mut self, size: u16) -> Option<BlockMetadata> {
        assert!(size as u64 >= self.free_blocks.list_node_size());
        assert_eq!(size % 8, 0);

        for block in self.free_blocks.iter() {
            let (header, ptr) = unsafe {
                let ptr = block.read().data;

                (ptr.read(), ptr)
            };

            if !header.is_used() && header.length() >= size {
                return Some(BlockMetadata {
                    header,
                    base: (ptr.as_ptr() as u64).as_virtual(),
                });
            }
        }

        if self.total_allocated + size < self.length() as u16 {
            let base = (self.get_base_address().as_addr() + self.total_allocated as u64)
                as *mut BlockHeader;

            unsafe { base.write(BlockHeader { data: size }) }

            self.total_allocated += 2 * size!(BlockHeader) as u16 + size;

            return unsafe { Some((*base).get_metadata()) };
        }

        None
    }
}
