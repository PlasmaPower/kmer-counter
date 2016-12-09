use errors::*;

#[derive(PartialEq, Eq)]
pub enum JoinMethod {
    Concat,
    Sort,
}

pub struct Leaf {
    pub counts: Vec<(u64, u16)>,
    pub sorted: bool,
}

impl Ord for Leaf {
    fn cmp(&self, other: &Leaf) -> Ordering {
        self.counts.map(|c| c.0).cmp(other.counts.map(|c| c.0)).rev()
    }
}

impl PartialOrd for Leaf {
    fn eq(&self, other: &Leaf) -> bool {
        // Compare in reverse as values are likely close
        self.counts.len() == other.counts.len() &&
        self.counts.map(|c| c.0).rev().eq(other.counts.map(|c| c.0).rev())
    }

    fn partial_cmp(&self, other: &Leaf) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub enum Node {
    Branch(Vec<Node>),
    Leaf(Leaf),
}

impl Node {
    pub fn consolidate<F>(self,
                          spawner: Spawner,
                          join_methods: Vec<JoinMethod>,
                          merge_dups: F)
                          -> Leaf
        where F: Fn(&K, &mut V, V) + Sync
    {
        let children = match *self {
            Leaf(leaf) => return leaf,
            Branch(children) => children,
        };
        let join_method = join_methods.pop();
        let counts = children.into_iter().map(|n| n.consolidate(join_methods.clone()));
        match join_method {
            Concat => {
                if let Some(first) = counts.next() {
                    let counts = counts.map(|n| n.counts);
                    Leaf {
                        counts: counts.fold(first.counts, |(mut a, b)| {
                            a.extend(b);
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
                let counts = counts.map(|n| {
                    if !n.sorted {
                        sort(n.counts, merge_dups, spawner);
                    }
                });
                // TODO: BinaryHeap sorting merger implementation
            }
        }
    }
}
