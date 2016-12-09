use std::collections::BinaryHeap;
use std::cmp::Ordering;

use jobsteal::Spawner;

use errors::*;
use sort::sort;

#[derive(PartialEq, Eq, Clone)]
pub enum JoinMethod {
    Concat,
    Sort,
}

pub struct Leaf {
    pub counts: Vec<Option<(u64, u16)>>,
    sorted: bool,
}

struct SortingQueueItem<'a> {
    counts: &'a [Option<(u64, u16)>],
}

impl<'a> SortingQueueItem<'a> {
    fn first(&self) -> Option<(u64, u16)> {
        self.counts.iter().filter_map(|o| *o).nth(0)
    }

    fn pop_first(&mut self) -> Option<&mut (u64, u16)> {
        loop {
            if let Some((next, rest)) = self.counts.split_first_mut() {
                self.counts = rest;
                if next.is_some() {
                    return next.as_mut();
                }
            } else {
                return None;
            }
        }
    }
}

/// Impl simply for Ord impl
impl<'a> PartialEq for SortingQueueItem<'a> {
    fn eq(&self, other: &SortingQueueItem) -> bool {
        match self.first() {
            None => other.first().is_none(),
            Some(a) => {
                match other.first() {
                    None => false,
                    Some(b) => a.0 == b.0,
                }
            }
        }
    }
}

impl<'a> Eq for SortingQueueItem<'a> {}

impl<'a> Ord for SortingQueueItem<'a> {
    fn cmp(&self, other: &SortingQueueItem) -> Ordering {
        match self.first() {
            None => {
                match other.first() {
                    None => Ordering::Equal,
                    Some(_) => Ordering::Less,
                }
            }
            Some(a) => {
                match other.first() {
                    None => Ordering::Greater,
                    Some(b) => a.0.cmp(&b.0).reverse(),
                }
            }
        }
    }
}

impl<'a> PartialOrd for SortingQueueItem<'a> {
    fn partial_cmp(&self, other: &SortingQueueItem) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub enum Node {
    Branch(Vec<Node>),
    Leaf(Leaf),
}

impl Node {
    pub fn consolidate<F>(self,
                          spawner: &Spawner,
                          join_methods: Vec<JoinMethod>,
                          merge_dups: &F)
                          -> Leaf
        where F: Fn(&u64, &mut u16, u16) + Sync
    {
        let children = match self {
            Node::Leaf(leaf) => return leaf,
            Node::Branch(children) => children,
        };
        let join_method = join_methods.pop();
        let children = children.into_iter().map(|n| n.consolidate(spawner, join_methods.clone(), merge_dups));
        match join_method {
            Concat => {
                if let Some(first) = children.next() {
                    let children = children.map(|n| n.counts);
                    Leaf {
                        counts: children.fold(first.counts, |mut a, b| {
                            a.append(&mut b);
                            a
                        }),
                        sorted: false,
                    }
                } else {
                    Leaf {
                        counts: Vec::new(),
                        sorted: true,
                    }
                }
            }
            Sort => {
                let output = Vec::new();
                let sorting_queue = children.map(|n| {
                        let counts = n.counts;
                        if !n.sorted {
                            sort(counts.as_mut_slice(), merge_dups, spawner);
                        }
                        SortingQueueItem { counts: counts.as_slice() }
                    })
                    .collect::<BinaryHeap<_>>();
                let mut next_count = sorting_queue.peek_mut().and_then(|item| item.pop_first());
                while let Some(&mut (kmer, mut count)) = next_count.take() {
                    while let Some(item) = sorting_queue.peek_mut() {
                        if let Some(first) = item.pop_first() {
                            if kmer == first.0 {
                                merge_dups(&kmer, &mut count, first.1);
                            } else {
                                next_count = Some(first);
                                break;
                            }
                        }
                    }
                    output.push(Some((kmer, count)));
                }
                Leaf {
                    counts: output,
                    sorted: true,
                }
            }
        }
    }
}
