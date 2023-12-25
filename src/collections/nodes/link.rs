use std::{
    ops::{Index, IndexMut, Deref, DerefMut},
    marker::PhantomData
};
use cc_traits::*;

use super::*;

#[const_trait]
pub trait Link: IndexMut<usize, Output = Ref<Self::Node>> + Default
{
    type Node: Node;
    fn count(&self) -> usize;
}
#[derive(Debug, PartialEq, Eq)]
pub struct Link1<T>(Ref<T>);
impl<T: Node> const Link for Link1<T> {
    type Node = T;
    #[inline(always)]
    fn count(&self) -> usize { 1 }
}
impl<T> const Default for Link1<T> {
    #[inline(always)]
    fn default() -> Self { Link1(None) }
}
impl<T> const Index<usize> for Link1<T> {
    type Output = Ref<T>;
    #[inline(always)]
    fn index(&self, _index: usize) -> &Self::Output { &self.0 }
}
impl<T> const IndexMut<usize> for Link1<T> {
    #[inline(always)]
    fn index_mut(&mut self, _index: usize) -> &mut Self::Output { &mut self.0 }
}
impl<T> const Deref for Link1<T> {
    type Target = Ref<T>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl<T> const DerefMut for Link1<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
impl<T> const From<Ref<T>> for Link1<T> {
    #[inline(always)]
    fn from(value: Ref<T>) -> Self { Link1(value) }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LinkWrapper<T, C>(C, PhantomData<T>);
impl<T: Node, C> const Link for LinkWrapper<T, C>
    where C: Len + Default + IndexMut<usize, Output = Ref<T>>
{
    type Node = T;
    #[inline(always)]
    fn count(&self) -> usize { self.0.len() }
}
impl<T, C> const Default for LinkWrapper<T, C>
    where C: Default
{
    #[inline(always)]
    fn default() -> Self { LinkWrapper(C::default(), PhantomData) }
}
impl<T, C> const Index<usize> for LinkWrapper<T, C>
    where C: Index<usize, Output = Ref<T>>
{
    type Output = Ref<T>;
    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output { &self.0[index] }
}
impl<T, C> const IndexMut<usize> for LinkWrapper<T, C>
    where C: IndexMut<usize, Output = Ref<T>>
{
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output { &mut self.0[index] }
}
impl<T, C> const Deref for LinkWrapper<T, C> {
    type Target = C;
    #[inline(always)]
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl<T, C> const DerefMut for LinkWrapper<T, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
impl<T, C> const From<C> for LinkWrapper<T, C> {
    #[inline(always)]
    fn from(value: C) -> Self { LinkWrapper(value, PhantomData) }
}

#[const_trait]
pub trait DirectedLink: Link + IndexMut<isize, Output = Ref<Self::Node>>
{
    fn lanes(&self) -> usize;
    #[inline(always)]
    fn count(&self) -> usize { self.lanes() << 1 }
}
// TODO: DirectedLink implementations

pub trait BranchLink: Link
{
    fn options(&self) -> usize;
    #[inline(always)]
    fn count(&self) -> usize { self.options() + 1 }
    #[inline(always)]
    fn parent(&self) -> &Ref<Self::Node> { &self[0] }
    #[inline(always)]
    fn parent_mut(&mut self) -> &mut Ref<Self::Node> { &mut self[0] }
    #[inline(always)]
    fn option(&self, index: usize) -> &Ref<Self::Node> { &self[index + 1] }
    #[inline(always)]
    fn option_mut(&mut self, index: usize) -> &Ref<Self::Node> { &mut self[index + 1] }
}
// TODO: BranchLink implementations