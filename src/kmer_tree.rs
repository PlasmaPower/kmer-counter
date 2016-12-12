use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::cmp::Ordering;

use jobsteal::Spawner;

use sort::sort;

#[derive(PartialEq, Eq, Clone)]
pub enum JoinMethod {
    Concat,
    Join,
    Sort,
}

pub struct Leaf {
    pub counts: Vec<Option<(u64, u16)>>,
    pub sorted: bool,
}

struct SortingQueueItem {
    counts: Vec<Option<(u64, u16)>>,
    index: usize,
}

// TODO: future optimization by collapsing trees
// e.g. no use in join -> sort, concat -> sort is quicker

impl SortingQueueItem {
    fn new(counts: Vec<Option<(u64, u16)>>) -> SortingQueueItem {
        SortingQueueItem {
            counts: counts,
            index: 0,
        }
    }

    fn first(&self) -> Option<&(u64, u16)> {
        self.counts[self.index..].iter().filter_map(|o| o.as_ref()).nth(0)
    }

    fn pop_first(&mut self) -> Option<(u64, u16)> {
        while let Some(mut count) = self.counts.get_mut(self.index) {
            self.index += 1;
            if count.is_some() {
                return count.take();
            }
        }
        None
    }
}

/// Impl simply for Ord impl
impl PartialEq for SortingQueueItem {
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

impl Eq for SortingQueueItem {}

impl Ord for SortingQueueItem {
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

impl PartialOrd for SortingQueueItem {
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
                          join_methods: &[JoinMethod],
                          merge_dups: &F)
                          -> Leaf
        where F: Fn(&u64, &mut u16, u16) + Sync
    {
        let children = match self {
            Node::Leaf(leaf) => return leaf,
            Node::Branch(children) => children,
        };
        let default_sort_method = JoinMethod::Concat; // Extends lifetime
        let join_methods_split = join_methods.split_first().unwrap_or((&default_sort_method, join_methods));
        let join_method = join_methods_split.0;
        let mut children = children.into_iter()
            .map(|n| n.consolidate(spawner, join_methods_split.1, merge_dups));
        match *join_method {
            JoinMethod::Concat => {
                if let Some(first) = children.next() {
                    let children = children.map(|n| n.counts);
                    Leaf {
                        counts: children.fold(first.counts, |mut a, mut b| {
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
            JoinMethod::Join => {
                let mut map = HashMap::new();
                for child in children {
                    for count in child.counts.into_iter().filter_map(|o| o) {
                        *map.entry(count.0).or_insert(0) += count.1;
                    }
                }
                Leaf {
                    counts: map.into_iter().map(Some).collect(),
                    sorted: false,
                }
            }
            JoinMethod::Sort => {
                let mut output = Vec::new();
                let mut sorting_queue = children.map(|n| {
                        let mut counts = n.counts;
                        if !n.sorted {
                            sort(Some(spawner), counts.as_mut_slice(), merge_dups);
                        }
                        SortingQueueItem::new(counts)
                    })
                    .collect::<BinaryHeap<_>>();
                let mut next_count = sorting_queue.peek_mut().and_then(|mut item| item.pop_first());
                let mut last = None;
                while let Some((kmer, count)) = next_count.take() {
                    if let Some(last) = last {
                        if kmer < last {
                            panic!("Encountered kmer {} after last {}", kmer, last);
                        }
                    }
                    last = Some(kmer);
                    let mut count = count;
                    while let Some(mut item) = sorting_queue.peek_mut() {
                        if let Some(first) = item.pop_first() {
                            if kmer == first.0 {
                                merge_dups(&kmer, &mut count, first.1);
                            } else {
                                next_count = Some(first);
                                break;
                            }
                        } else {
                            break;
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
