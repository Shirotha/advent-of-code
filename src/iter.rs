use std::collections::VecDeque;

enum DFSState {
    Done,
    Branch,
    Leaf,
    Backtrack
}

pub struct DFSIter<N, I, F> {
    neighbours: F,
    stack: VecDeque<I>,
    current: Option<N>
}
impl<N, I, F> DFSIter<N, I, F>
    where F: FnMut(&N) -> Option<I>
{
    #[inline]
    pub fn new(mut neighbours: F, root: N) -> Self {
        // ASSERT: neighbours is a graph without cycles
        let mut stack = VecDeque::new();
        stack.push_back(neighbours(&root).unwrap());
        Self { neighbours, stack, current: Some(root) }
    }
}
impl<N, I, F> DFSIter<N, I, F>
where
    I: Iterator<Item = N>,
    F: FnMut(&N) -> Option<I>
{
    #[inline]
    fn step(&mut self) -> DFSState {
        if let Some(iter) = self.stack.back_mut() {
            if let Some(node) = iter.next() {
                if let Some(neighbours) = (self.neighbours)(&node) {
                    self.stack.push_back(neighbours);
                    self.current = Some(node);
                    DFSState::Branch
                } else {
                    self.current = Some(node);
                    DFSState::Leaf
                }
            } else {
                self.stack.pop_back();
                self.current = None;
                DFSState::Backtrack
            }
        } else {
            self.current = None;
            DFSState::Done
        }
    }
}
impl<N, I, F> Iterator for DFSIter<N, I, F>
where
    I: Iterator<Item = N>,
    F: FnMut(&N) -> Option<I>
{
    type Item = N;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take();
        while matches!(self.step(), DFSState::Backtrack) {}
        current
    }
}

enum PathState {
    Done,
    Walking,
    Path
}

pub struct PathIter<N, I, F> {
    dfs: DFSIter<N, I, F>,
    path: Vec<N>
}
impl<N: Clone, I, F> PathIter<N, I, F>
    where F: FnMut(&N) -> Option<I>
{
    #[inline]
    pub fn new(neighbours: F, root: N) -> Self {
        Self { dfs: DFSIter::new(neighbours, root.clone()), path: vec![root] }
    }
}
impl<N: Clone, I, F> PathIter<N, I, F> {
    #[inline]
    fn path(&mut self) -> Vec<N> {
        let path = self.path.clone();
        self.path.pop();
        path
    }
}
impl<N, I, F> PathIter<N, I, F>
where
    I: Iterator<Item = N>,
    F: FnMut(&N) -> Option<I>
{
    #[inline]
    fn step(&mut self) -> PathState {
        match self.dfs.step() {
            DFSState::Done => PathState::Done,
            DFSState::Branch => {
                self.path.push(self.dfs.current.take().unwrap());
                PathState::Walking
            },
            DFSState::Leaf => {
                self.path.push(self.dfs.current.take().unwrap());
                PathState::Path
            },
            DFSState::Backtrack => {
                self.path.pop();
                PathState::Walking
            }
        }
    }
}
impl<'a, N: Clone, I, F> Iterator for &'a mut PathIter<N, I, F>
where
    I: Iterator<Item = N>,
    F: FnMut(&N) -> Option<I>
{
    type Item = Vec<N>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.step() {
                PathState::Done => return None,
                PathState::Walking => (),
                PathState::Path => return Some(self.path())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn dfs_iter() {
        let edges = vec![
            vec![1, 2],
            vec![3],
            vec![4, 5],
            vec![4],
            vec![],
            vec![0],
        ];
        let neighbours = |node: &i32| {
            static mut VISITED: Vec<bool> = Vec::new();
            unsafe {
                if VISITED.is_empty() {
                    VISITED.resize(edges.len(), false);
                }
                let i = *node as usize;
                VISITED[i] = true;
                edges.get(i)
                    .map( |neighbours| neighbours.iter().copied().filter( |node| !VISITED[*node as usize] ) )
            }
        };
        let iter = DFSIter::new(neighbours, 0);
        assert_eq!(iter.collect_vec(), vec![0, 1, 3, 4, 2, 5]);
    }

    #[test]
    fn path_iter() {
        let edges = vec![
            vec![1, 2],
            vec![3],
            vec![4, 5],
            vec![4],
            vec![],
            vec![],
        ];
        let neighbours = |node: &i32| {
            edges.get(*node as usize)
                .filter( |neighbours| !neighbours.is_empty() )
                .map( |neighbours| neighbours.iter().copied() )
        };
        let mut iter = PathIter::new(neighbours, 0);
        assert_eq!(iter.collect_vec(), vec![
            vec![0, 1, 3, 4],
            vec![0, 2, 4],
            vec![0, 2, 5]
        ]);
    }
}