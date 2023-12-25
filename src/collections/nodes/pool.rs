use super::*;

use std::{
    ptr::NonNull,
    alloc::{Allocator, Layout, AllocError, LayoutError, Global}
};

use tap::Pipe;
#[easy_ext::ext]
impl<T: Node> T {
    #[inline(always)]
    fn pool_link(&mut self) -> &mut Ref<Self> {
        &mut self.link_mut()[0]
    }
}

#[derive(Debug)]
pub struct Pool<T: Node, A: Allocator> {
    head: Ref<T>,
    allocator: A,
    layout: Layout,
}
impl<T: Node> Pool<T, Global> {
    #[inline(always)]
    pub fn new() -> Result<Self, LayoutError> {
        Self::with_allocator(Global)
    }
}
impl<T: Node, A: Allocator> Pool<T, A> {
    #[inline]
    pub fn with_allocator(allocator: A) -> Result<Self, LayoutError> {
        let layout = Layout::new::<T>();
        Self { head: None, allocator, layout }.pipe(Ok)
    }
    #[inline]
    pub fn get(&mut self) -> Result<NonNull<T>, AllocError> {
        let ptr = match self.head {
            Some(mut ptr) => {
                let node = unsafe { ptr.as_mut() };
                self.head = node.pool_link().take();
                ptr
            },
            None => {
                self.allocator.allocate(self.layout)?.cast()
            }
        };
        unsafe { ptr.write(T::default()); }
        Ok(ptr)
    }
    #[inline]
    pub fn put(&mut self, mut node: NonNull<T>) {
        // ASSERT: node does not own memory, or is linking to other nodes
        unsafe {
            debug_assert!( node.as_ref().link().pipe( |link| (0..link.count()).all( |i| link[i].is_none() ) ) );
            *node.as_mut().pool_link() = self.head;
        }
        self.head = Some(node);
    }
}
impl<T: Node, A: Allocator> Drop for Pool<T, A> {
    #[inline]
    fn drop(&mut self) {
        while let Some(mut ptr) = self.head {
            unsafe {
                self.head = ptr.as_mut().pool_link().take();
                self.allocator.deallocate(ptr.cast(), self.layout);
            }
        }
    }
}