use std::cmp::Ordering;

use std::fmt::Debug;

use jobsteal::Pool;
use jobsteal::Spawner;

trait Joiner {
    type NewJoiner: Joiner;

    fn is_parallel() -> bool;
    fn join<A, RA, B, RB>(&self, oper_a: A, oper_b: B) -> (RA, RB)
        where A: FnOnce(Self::NewJoiner) -> RA + Send,
              B: FnOnce(Self::NewJoiner) -> RB + Send,
              RA: Send,
              RB: Send,
              Self: Sized;
}

struct Parallel<'s, 'p: 's> {
    spawner: &'s Spawner<'p, 's>,
}

impl<'s, 'p, 'n> Joiner for Parallel<'s, 'p> {
    type NewJoiner = Parallel<'s, 'n>;

    #[inline]
    fn is_parallel() -> bool {
        true
    }
    #[inline]
    fn join<A, RA, B, RB>(&self, oper_a: A, oper_b: B) -> (RA, RB)
        where A: FnOnce(Parallel<'s, 'n>) -> RA + Send,
              B: FnOnce(Parallel<'s, 'n>) -> RB + Send,
              RA: Send,
              RB: Send
    {
        self.spawner.join(|s| {
                              let joiner = Parallel { spawner: s };
                              oper_a(joiner)
                          },
                          |s| {
                              let joiner = Parallel { spawner: s };
                              oper_b(joiner)
                          })
    }
}

struct Sequential;
impl Joiner for Sequential {
    type NewJoiner = Sequential;

    #[inline]
    fn is_parallel() -> bool {
        false
    }
    #[inline]
    fn join<A, RA, B, RB>(&self, oper_a: A, oper_b: B) -> (RA, RB)
        where A: FnOnce(Self) -> RA,
              B: FnOnce(Self) -> RB
    {
        let a = oper_a(Sequential {});
        let b = oper_b(Sequential {});
        (a, b)
    }
}

fn quick_sort<J: Joiner,
              K: Ord + Clone + Send + Debug,
              V: Send + Debug,
              F: Fn(&K, &mut V, V) + Sync>
    (v: &mut [Option<(K, V)>],
     merge_dups: &F,
     joiner: J) {
    if v.len() <= 1 {
        return;
    }

    if J::is_parallel() && v.len() <= 5 * 1024 {
        return quick_sort(v, merge_dups, Sequential {});
    }

    let split_point = partition(v, merge_dups);
    let (lo, hi) = v.split_at_mut(split_point);
    joiner.join(|j| quick_sort(lo, merge_dups, j),
                |j| quick_sort(hi, merge_dups, j));
}

pub fn sort<K: Ord + Clone + Send + Debug,
            V: Send + Debug,
            F: Fn(&K, &mut V, V) + Sync>(v: &mut [Option<(K, V)>], merge_dups: F, pool: Pool) {
    quick_sort(v, &merge_dups, Parallel { spawner: &pool.spawner() })
}

fn compare<K: Ord, V>(a: &Option<(K, V)>, b: &Option<(K, V)>) -> Ordering {
    match *a {
        Some(ref a) => {
            match *b {
                Some(ref b) => a.0.cmp(&b.0),
                None => Ordering::Greater,
            }
        }
        None => Ordering::Less,
    }
}

fn partition<K: Ord + Clone + Send + Debug, V: Send + Debug, F: Fn(&K, &mut V, V) + Sync>
    (v: &mut [Option<(K, V)>],
     merge_dups: &F)
     -> usize {
    let pivot = v.len() - 1;
    debug!("Partitioning {:?}", v);
    let mut i = 0;
    for j in 0..pivot {
        let order = compare(&v[j], &v[pivot]);
        if order == Ordering::Equal {
            let old = v[j].take();
            if let Some(&mut Some(ref mut pivot)) = v.get_mut(pivot) {
                let old = old.unwrap();
                merge_dups(&old.0, &mut pivot.1, old.1);
            }
        }
        if order != Ordering::Greater {
            v.swap(i, j);
            i += 1;
        }
        debug!("Partition iteration done: i={}, j={}, v={:?}", i, j, v);
    }
    v.swap(i, pivot);
    debug!("After partition at {}: {:?}", i, v);
    i
}
