mod link;
mod pool;
mod cursor;

pub use link::*;
pub use pool::*;
pub use cursor::*;

use std::{ptr::NonNull, marker::PhantomData};

pub type Ref<T> = Option<NonNull<T>>;

#[const_trait]
pub trait Node: Default {
    type Link: Link<Node = Self>;
    fn link(&self) -> &Self::Link;
    fn link_mut(&mut self) -> &mut Self::Link;
}

macro_rules! node {
    { $name:ident ( $data:ty , $link:ty ) { $default:expr } } => {
        #[derive(Debug, PartialEq, Eq)]
        struct $name {
            pub data: $data,
            link: $link
        }
        impl $name {
            #[inline(always)]
            fn new(data: $data) -> Self { $name { data, link: <$link>::default() } }
        }
        impl const Node for $name {
            type Link = $link;
            #[inline(always)]
            fn link(&self) -> &Self::Link { &self.link }
            #[inline(always)]
            fn link_mut(&mut self) -> &mut Self::Link { &mut self.link }
        }
        impl const Default for $name {
            #[inline(always)]
            fn default() -> Self { Self::new($default) }
        }
    };
    { $name:ident < $( $generics:tt ),+ > ( $data:ty , $link:ty ) { $default:expr } } => {
        #[derive(Debug, PartialEq, Eq)]
        struct $name < $( $generics ),* > {
            data: $data,
            link: $link
        }
        impl< $( $generics ),* > $name < $( $generics ),* > {
            #[inline(always)]
            fn new(data: $data) -> Self { $name { data, link: <$link>::default() } }
        }
        impl< $( $generics ),* > const Node for $name < $( $generics ),* > {
            type Link = $link;
            #[inline(always)]
            fn link(&self) -> &Self::Link { &self.link }
            #[inline(always)]
            fn link_mut(&mut self) -> &mut Self::Link { &mut self.link }
        }
        impl< $( $generics ),* > const Default for $name < $( $generics ),* > {
            #[inline(always)]
            fn default() -> Self { Self::new($default) }
        }
    };
}
pub(crate) use node;

#[derive(Debug)]
pub struct LinkIter<'a, T>
{
    current: Ref<T>,
    index: usize,
    phantom: PhantomData<&'a ()>
}
impl<'a, T: Node + 'a> Iterator for LinkIter<'a, T>
{
    type Item = &'a T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Some(ptr) => {
                let node = unsafe { ptr.as_ref() };
                self.current = node.link()[self.index];
                Some(node)
            },
            None => None
        }
    }
}
#[easy_ext::ext]
pub impl<T: Node> Ref<T> {
    #[inline(always)]
    fn iter_link(&self, index: usize) -> LinkIter<T> {
        LinkIter { current: *self, index, phantom: PhantomData }
    }
}

#[derive(Debug)]
pub struct LinkIterMut<'a, T>
{
    current: Ref<T>,
    index: usize,
    phantom: PhantomData<&'a ()>
}
impl<'a, T: Node + 'a> Iterator for LinkIterMut<'a, T>
{
    type Item = &'a mut T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Some(mut ptr) => {
                let node = unsafe { ptr.as_mut() };
                self.current = node.link()[self.index];
                Some(node)
            },
            None => None
        }
    }
}

#[easy_ext::ext]
pub impl<T: Node> Ref<T> {
    #[inline(always)]
    fn iter_link_mut(&mut self, index: usize) -> LinkIterMut<T> {
        LinkIterMut { current: *self, index, phantom: PhantomData }
    }
}