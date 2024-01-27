use core::ptr::NonNull;

pub struct DoublyLinkedListNode<T> {
    pub next: Option<NonNull<DoublyLinkedListNode<T>>>,
    pub prev: Option<NonNull<DoublyLinkedListNode<T>>>,
    pub data: T,
}

pub struct DoublyLinkedList<T> {
    pub head: Option<NonNull<DoublyLinkedListNode<T>>>,
    pub tail: Option<NonNull<DoublyLinkedListNode<T>>>,
}

pub struct SinglyLinkedListNode<T> {
    pub next: Option<NonNull<SinglyLinkedListNode<T>>>,
    pub data: T,
}

pub struct SinglyLinkedList<T> {
    head: Option<NonNull<SinglyLinkedListNode<T>>>,
    tail: Option<NonNull<SinglyLinkedListNode<T>>>,
}

impl<T> DoublyLinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    pub fn tail(&mut self) -> Option<&T> {
        match self.tail {
            None => None,
            Some(mut tail) => unsafe { Some(&tail.as_mut().data) },
        }
    }

    pub fn head(&mut self) -> Option<&T> {
        match self.head {
            None => None,
            Some(mut head) => unsafe { Some(&head.as_mut().data) },
        }
    }

    pub fn tail_mut(&mut self) -> Option<&mut T> {
        match self.tail {
            None => None,
            Some(mut tail) => unsafe { Some(&mut tail.as_mut().data) },
        }
    }

    pub fn head_mut(&mut self) -> Option<&mut T> {
        match self.head {
            None => None,
            Some(mut head) => unsafe { Some(&mut head.as_mut().data) },
        }
    }

    pub unsafe fn append(&mut self, elem: T, ptr: *mut DoublyLinkedListNode<T>) {
        ptr.write(DoublyLinkedListNode {
            data: elem,
            next: None,
            prev: None,
        });

        let mut new_node = NonNull::new_unchecked(ptr);

        if self.head.is_none() {
            self.head = Some(new_node);
            self.tail = Some(new_node);
            return;
        }

        if let Some(mut node) = self.tail {
            new_node.as_mut().prev = self.tail;
            new_node.as_mut().next = None;

            node.as_mut().next = Some(new_node);

            self.tail = Some(new_node);
        }
    }

    pub unsafe fn remove(
        &mut self,
        compare: impl Fn(&T) -> bool,
        free_function: unsafe fn(*mut DoublyLinkedListNode<T>),
    ) {
        let mut node = self.head;

        while let Some(mut node_ptr) = node {
            if compare(&node_ptr.as_mut().data) {
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

            free_function(node_ptr.as_ptr());
            break;
        }
    }
}

impl<T> SinglyLinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    pub fn tail(&mut self) -> Option<&T> {
        match self.tail {
            None => None,
            Some(mut tail) => unsafe { Some(&tail.as_mut().data) },
        }
    }

    pub fn head(&mut self) -> Option<&T> {
        match self.head {
            None => None,
            Some(mut head) => unsafe { Some(&head.as_mut().data) },
        }
    }

    pub fn tail_mut(&mut self) -> Option<&mut T> {
        match self.tail {
            None => None,
            Some(mut tail) => unsafe { Some(&mut tail.as_mut().data) },
        }
    }

    pub fn head_mut(&mut self) -> Option<&mut T> {
        match self.head {
            None => None,
            Some(mut head) => unsafe { Some(&mut head.as_mut().data) },
        }
    }

    pub unsafe fn append(&mut self, data: T, ptr: *mut SinglyLinkedListNode<T>) {
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
    }

    pub unsafe fn remove(
        &mut self,
        compare: impl Fn(&T) -> bool,
        free_function: unsafe fn(*mut SinglyLinkedListNode<T>),
    ) {
        let mut node = self.head;
        let mut previous_node = None;

        while let Some(mut node_ptr) = node {
            if compare(&node_ptr.as_mut().data) {
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

            free_function(node_ptr.as_ptr());
        }
    }
}
