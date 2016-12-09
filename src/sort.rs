use std::cmp::Ordering;
use std::fmt::Debug;

use jobsteal::Spawner;

fn quick_sort<K: Ord + Clone + Send + Debug,
              V: Send + Debug,
              F: Fn(&K, &mut V, V) + Sync>
    (v: &mut [Option<(K, V)>],
     merge_dups: &F,
     spawner: Option<&Spawner>) {
    if v.len() <= 1 {
        return;
    }

    if spawner.is_some() && v.len() <= 5 * 1024 {
        return quick_sort(v, merge_dups, None);
    }

    let split_point = partition(v, merge_dups);
    let (lo, hi) = v.split_at_mut(split_point);

    if let Some(spawner) = spawner {
        spawner.join(|j| quick_sort(lo, merge_dups, Some(j)),
                     |j| quick_sort(hi, merge_dups, Some(j)));
    } else {
        quick_sort(lo, merge_dups, None);
        quick_sort(hi, merge_dups, None);
    }
}

pub fn sort<K: Ord + Clone + Send + Debug,
            V: Send + Debug,
            F: Fn(&K, &mut V, V) + Sync>(v: &mut [Option<(K, V)>], merge_dups: F, spawner: &Spawner) {
    quick_sort(v, &merge_dups, Some(spawner))
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
