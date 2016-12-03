use std::cmp::Ordering;

use std::fmt::Debug;

use rayon;

trait Joiner {
    fn is_parallel() -> bool;
    fn join<A, RA, B, RB>(oper_a: A, oper_b: B) -> (RA, RB)
        where A: FnOnce() -> RA + Send,
              B: FnOnce() -> RB + Send,
              RA: Send,
              RB: Send;
}

struct Parallel;
impl Joiner for Parallel {
    #[inline]
    fn is_parallel() -> bool {
        true
    }
    #[inline]
    fn join<A, RA, B, RB>(oper_a: A, oper_b: B) -> (RA, RB)
        where A: FnOnce() -> RA + Send,
              B: FnOnce() -> RB + Send,
              RA: Send,
              RB: Send
              {
                  rayon::join(oper_a, oper_b)
              }
}

struct Sequential;
impl Joiner for Sequential {
    #[inline]
    fn is_parallel() -> bool {
        false
    }
    #[inline]
    fn join<A, RA, B, RB>(oper_a: A, oper_b: B) -> (RA, RB)
        where A: FnOnce() -> RA,
              B: FnOnce() -> RB
              {
                  let a = oper_a();
                  let b = oper_b();
                  (a, b)
              }
}

fn quick_sort<J: Joiner, K: Ord + Clone + Send + Debug, V: Send + Debug, F: Fn(&K, &mut V, V) + Sync>(v: &mut [Option<(K, V)>],
                                                                                                      merge_dups: &F) {
    if v.len() <= 1 {
        return;
    }

    if J::is_parallel() && v.len() <= 5 * 1024 {
        return quick_sort::<Sequential, K, V, F>(v, merge_dups);
    }

    let split_point = partition(v, merge_dups);
    let (lo, hi) = v.split_at_mut(split_point);
    J::join(|| quick_sort::<J, K, V, F>(lo, merge_dups),
    || quick_sort::<J, K, V, F>(hi, merge_dups));
}

pub fn sort<K: Ord + Clone + Send + Debug, V: Send + Debug, F: Fn(&K, &mut V, V) + Sync>(v: &mut [Option<(K, V)>], merge_dups: F) {
    quick_sort::<Parallel, K, V, F>(v, &merge_dups)
}

fn compare<K: Ord, V>(a: &Option<(K, V)>, b: &Option<(K, V)>) -> Ordering {
    match *a {
        Some(ref a) => match *b {
            Some(ref b) => a.0.cmp(&b.0),
            None => Ordering::Greater,
        },
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
