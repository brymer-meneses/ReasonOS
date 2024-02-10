use core::fmt;
use core::ptr::addr_of_mut;
use core::ptr::NonNull;

use crate::memory::VirtualAddress;
use crate::misc::utils::size;

#[repr(C)]
#[derive(Debug)]
pub struct DoublyLinkedList<T> {
    head: Option<NonNull<DoublyLinkedListNode<T>>>,
    tail: Option<NonNull<DoublyLinkedListNode<T>>>,
    pub length: usize,
}

#[repr(C)]
pub struct SinglyLinkedList<T> {
    head: Option<NonNull<SinglyLinkedListNode<T>>>,
    tail: Option<NonNull<SinglyLinkedListNode<T>>>,
    pub length: usize,
}

#[repr(C, align(8))]
#[derive(Debug)]
pub struct DoublyLinkedListNode<T> {
    pub next: Option<NonNull<DoublyLinkedListNode<T>>>,
    pub prev: Option<NonNull<DoublyLinkedListNode<T>>>,
    pub data: T,
}

#[repr(C, align(8))]
#[derive(Debug)]
pub struct SinglyLinkedListNode<T> {
    pub next: Option<NonNull<SinglyLinkedListNode<T>>>,
    pub data: T,
}

impl<T> DoublyLinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            length: 0,
        }
    }

    pub unsafe fn append_to_address(&mut self, address: VirtualAddress, data: T) {
        let ptr = address.as_addr() as *mut DoublyLinkedListNode<T>;
        ptr.write(DoublyLinkedListNode {
            data,
            next: None,
            prev: None,
        });

        let mut new_node = NonNull::new_unchecked(ptr);

        match self.tail {
            None => {
                self.head = Some(new_node);
                self.tail = Some(new_node);
            }
            Some(mut node) => {
                new_node.as_mut().prev = self.tail;
                new_node.as_mut().next = None;

                node.as_mut().next = Some(new_node);

                self.tail = Some(new_node);
            }
        }

        self.length += 1;
    }

    pub unsafe fn remove(&mut self, compare: impl Fn(&mut DoublyLinkedListNode<T>) -> bool) {
        assert_ne!(self.length, 0);

        let mut node = self.head;

        while let Some(mut node_ptr) = node {
            if compare(node_ptr.as_mut()) {
                node = node_ptr.as_mut().next;
                continue;
            }

            let previous_node = node_ptr.as_mut().prev;
            let next_node = node_ptr.as_mut().next;

            match (previous_node, next_node) {
                (None, None) => {
                    self.head = None;
                    self.tail = None;
                }
                (Some(mut prev), Some(mut next)) => {
                    prev.as_mut().next = next_node;
                    next.as_mut().prev = previous_node;
                }
                // at the start of the list
                (None, Some(_)) => {
                    self.head = next_node;
                    self.head.unwrap().as_mut().prev = None;
                }
                // at the end of the list
                (Some(_), None) => {
                    self.tail = previous_node;
                    self.tail.unwrap().as_mut().next = None;
                }
            }

            self.length -= 1;
            break;
        }
    }

    pub fn iter(&self) -> DoublyLinkedListIterator<T> {
        DoublyLinkedListIterator {
            list: self,
            current: self.head,
        }
    }

    pub fn tail(&mut self) -> Option<NonNull<T>> {
        match self.tail {
            None => None,
            Some(mut tail) => unsafe { Some(tail.as_mut().ptr_to_data()) },
        }
    }

    pub fn head(&mut self) -> Option<NonNull<T>> {
        match self.head {
            None => None,
            Some(mut head) => unsafe { Some(head.as_mut().ptr_to_data()) },
        }
    }

    pub fn tail_node(&mut self) -> Option<NonNull<DoublyLinkedListNode<T>>> {
        return self.head;
    }

    pub fn head_node(&mut self) -> Option<NonNull<DoublyLinkedListNode<T>>> {
        return self.head;
    }

    pub const fn list_node_size(&self) -> u64 {
        size!(DoublyLinkedListNode<T>)
    }

    pub const fn list_ptr_size(&self) -> u64 {
        size!(Option<NonNull<DoublyLinkedListNode<T>>>)
    }
}

impl<T> SinglyLinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            length: 0,
            tail: None,
        }
    }

    /// # Safety:
    /// - Ensure that the `address` passed has enough capacity for
    /// `size_of<SinglyLinkedListNode<T>`
    pub unsafe fn append_to_address(&mut self, address: VirtualAddress, data: T) {
        let ptr = address.as_addr() as *mut SinglyLinkedListNode<T>;
        ptr.write(SinglyLinkedListNode { data, next: None });
        let node = Some(NonNull::new_unchecked(ptr));

        match self.tail {
            None => {
                self.tail = node;
                self.head = node;
            }

            Some(mut tail) => {
                tail.as_mut().next = node;
                self.tail = node;
            }
        };

        self.length += 1;
    }

    pub unsafe fn remove(&mut self, compare: impl Fn(&mut SinglyLinkedListNode<T>) -> bool) {
        assert_ne!(self.length, 0);
        let mut node = self.head;
        let mut previous_node = None;

        while let Some(mut node_ptr) = node {
            if compare(node_ptr.as_mut()) {
                previous_node = node;
                node = node_ptr.as_mut().next;
                continue;
            }

            let next_node = node_ptr.as_mut().next;
            match (previous_node, next_node) {
                (None, None) => {
                    self.head = None;
                    self.tail = None;
                }
                (Some(mut prev_node_ptr), Some(_)) => {
                    prev_node_ptr.as_mut().next = next_node;
                }
                // end of the list
                (Some(mut prev_node_ptr), None) => {
                    prev_node_ptr.as_mut().next = None;
                    self.tail = previous_node;
                }
                // beginning of the list
                (None, Some(_)) => {
                    self.head = next_node;
                }
            }

            self.length -= 1;
            break;
        }
    }

    pub fn iter(&self) -> SinglyLinkedListIterator<T> {
        SinglyLinkedListIterator {
            list: self,
            current: self.head,
        }
    }

    pub fn tail(&mut self) -> Option<NonNull<T>> {
        match self.tail {
            None => None,
            Some(mut tail) => unsafe {
                Some(NonNull::new_unchecked(addr_of_mut!(tail.as_mut().data)))
            },
        }
    }

    pub fn head(&self) -> Option<NonNull<T>> {
        match self.head {
            None => None,
            Some(mut head) => unsafe {
                Some(NonNull::new_unchecked(addr_of_mut!(head.as_mut().data)))
            },
        }
    }

    pub fn tail_node(&mut self) -> Option<NonNull<SinglyLinkedListNode<T>>> {
        return self.head;
    }

    pub fn head_node(&mut self) -> Option<NonNull<SinglyLinkedListNode<T>>> {
        return self.head;
    }

    pub const fn list_node_size(&self) -> u64 {
        size!(SinglyLinkedListNode<T>)
    }

    pub const fn list_ptr_size(&self) -> u64 {
        size!(Option<NonNull<SinglyLinkedListNode<T>>>)
    }
}

impl<T> SinglyLinkedListNode<T> {
    pub unsafe fn ptr_to_data(&mut self) -> NonNull<T> {
        NonNull::new_unchecked(addr_of_mut!(self.data))
    }
}

impl<T> DoublyLinkedListNode<T> {
    pub unsafe fn ptr_to_data(&mut self) -> NonNull<T> {
        NonNull::new_unchecked(addr_of_mut!(self.data))
    }
}

pub struct DoublyLinkedListIterator<'a, T> {
    list: &'a DoublyLinkedList<T>,
    current: Option<NonNull<DoublyLinkedListNode<T>>>,
}

pub struct SinglyLinkedListIterator<'a, T> {
    list: &'a SinglyLinkedList<T>,
    current: Option<NonNull<SinglyLinkedListNode<T>>>,
}

impl<'a, T> Iterator for DoublyLinkedListIterator<'a, T> {
    type Item = NonNull<DoublyLinkedListNode<T>>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            None => None,
            Some(mut node_ptr) => {
                self.current = unsafe { node_ptr.as_mut().next };
                Some(node_ptr)
            }
        }
    }
}

impl<'a, T> Iterator for SinglyLinkedListIterator<'a, T> {
    type Item = NonNull<SinglyLinkedListNode<T>>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            None => None,
            Some(mut node_ptr) => {
                self.current = unsafe { node_ptr.as_mut().next };
                Some(node_ptr)
            }
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for SinglyLinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut node = self.head;

        if node.is_none() {
            return write!(f, "{{}}");
        }

        while let Some(node_ptr) = node {
            write!(f, ",\n")?;
            <SinglyLinkedListNode<T> as fmt::Debug>::fmt(unsafe { node_ptr.as_ref() }, f)?;
            node = unsafe { node_ptr.as_ref().next };
        }

        Ok(())
    }
}
