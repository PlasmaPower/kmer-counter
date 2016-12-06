use std::convert;

use count;
use sort::sort;

use memchr::memchr;
use rayon::prelude::*;

fn get_genome_slices(input_slice: &[u8]) -> Vec<&[u8]> {
    let mut slices = vec![input_slice];
    while let Some(slice) = slices.pop() {
        if let Some(index) = memchr(b'>', slice) {
            if index != 0 {
                slices.push(&slice[..index]);
            }
            if let Some(newline_index) = memchr(b'\n', &slice[(index + 1)..]) {
                slices.push(&slice[(index + 1 + newline_index + 1)..]);
            } else {
                break;
            }
        } else {
            slices.push(slice);
            break;
        }
    }
    return slices;
}

pub struct Options {
    pub kmer_len: u8,
    pub only_presence: bool,
}

enum JoinTransformStage<O, T> {
    Base(Vec<O>),
    Transformed(Vec<T>),
}

impl<O, T> JoinTransformStage<O, T>
    where O: Into<T>
{
    fn transform(self) -> Vec<T> {
        match self {
            JoinTransformStage::Base(o) => o.into_iter().map(convert::Into::into).collect(),
            JoinTransformStage::Transformed(t) => t,
        }
    }
}

pub fn run_counts(input_slice: &[u8], opts: &Options) -> Vec<Option<(u64, u16)>> {
    let genome_slices = get_genome_slices(input_slice);
    info!("{} genome slices found", genome_slices.len());

    let kmer_bitmask = if opts.kmer_len < 32 {
        (1 << (2 * (opts.kmer_len as u64))) - 1
    } else {
        // We don't want it to overflow
        ::std::u64::MAX
    };
    let count_options = count::Options {
        kmer_len: opts.kmer_len,
        kmer_bitmask: kmer_bitmask,
    };
    let mut counts: Vec<Option<(u64, u16)>> = genome_slices.into_par_iter()
        // Count each slice
        // Converts to an option for sorting
        .map(|slice| count::count(slice, &count_options))
        .map(JoinTransformStage::Base)
        // Merge the resultant counts
        .reduce(|| JoinTransformStage::Transformed(Vec::new()), |a, b| {
            let mut a = a.transform();
            a.extend(b.transform());
            JoinTransformStage::Transformed(a)
        })
        .transform();
    if opts.only_presence {
        sort(counts.as_mut_slice(), |_, _, _| {})
    } else {
        sort(counts.as_mut_slice(), |_, value, other| *value += other)
    };
    counts
}
