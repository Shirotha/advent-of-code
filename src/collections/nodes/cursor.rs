use std::alloc::{Allocator, AllocError};

use super::*;
// TODO: different implementation for DirectedLink (/ BranchLink)
pub struct Cursor<'a, T> {
    current: &'a mut Option<NonNull<T>>,
}
impl<'a, T: Node> Cursor<'a, T> {
    #[inline]
    pub fn move_link(&mut self, index: usize) -> bool {
        if let Some(ptr) = self.current {
            let node = unsafe { ptr.as_mut() };
            self.current = &mut node.link_mut()[index];
            true
        } else { false }
    }
    #[inline]
    pub fn current(&self) -> Option<&T> {
        self.current.map( |ptr| unsafe { ptr.as_ref() } )
    }
    #[inline]
    pub fn current_mut(&mut self) -> Option<&mut T> {
        self.current.map( |mut ptr| unsafe { ptr.as_mut() } )
    }
    #[allow(dead_code)]
    #[inline]
    pub fn peek(&self, index: usize) -> Option<&T> {
        self.current().and_then( |node| 
            node.link()[index].map( |ptr| unsafe { ptr.as_ref() } )
        )
    }
    #[inline]
    pub fn insert<A: Allocator>(&mut self, index: usize, pool: &mut Pool<T, A>) -> Result<&mut T, AllocError> {
        let mut ptr = pool.get()?;
        if let Some(prev_ptr) = self.current {
            unsafe {
                let next_ptr = prev_ptr.as_mut().link()[index];
                ptr.as_mut().link_mut()[index] = next_ptr;
            }
        }
        *self.current = Some(ptr);
        Ok(unsafe { ptr.as_mut() })
    }
    #[inline]
    pub fn remove<A: Allocator>(&mut self, index: usize, pool: &mut Pool<T, A>) -> bool {
        // ASSERT: current does not own memory, or is linking any node other than index
        if let Some(mut ptr) = *self.current {
            *self.current = unsafe { ptr.as_mut().link_mut()[index].take() };
            pool.put(ptr);
            true
        } else { false }
    }
}
#[easy_ext::ext]
impl<T: Node> Ref<T> {
    #[inline(always)]
    pub fn cursor(&mut self) -> Cursor<T> { Cursor { current: self } }
}