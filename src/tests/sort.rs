use sort::*;

const SORT_INPUT: [(u8, u8); 8] = [(0b11100001, 1),
                                   (0b01011000, 1),
                                   (0b01111100, 1),
                                   (0b10100100, 1),
                                   (0b10000001, 1),
                                   (0b01111100, 2),
                                   (0b00010100, 1),
                                   (0b00010100, 2)];

#[test]
fn simple_sort() {
    ::env_logger::init().ok();

    let mut input = SORT_INPUT.iter().cloned().map(Some).collect::<Vec<_>>();
    sort(input.as_mut_slice(), |_, _, _| {});
    assert_eq!(input.into_iter().filter_map(|n| n).map(|n| n.0).collect::<Vec<_>>(),
               vec![0b00010100, 0b01011000, 0b01111100, 0b10000001, 0b10100100, 0b11100001]);
}

#[test]
fn merge_dups() {
    ::env_logger::init().ok();

    let mut input = SORT_INPUT.iter().cloned().map(Some).collect::<Vec<_>>();
    sort(input.as_mut_slice(), |_, acc, other| *acc += other);
    assert_eq!(input.into_iter().filter_map(|n| n).collect::<Vec<_>>(),
               vec![(0b00010100, 3),
                    (0b01011000, 1),
                    (0b01111100, 3),
                    (0b10000001, 1),
                    (0b10100100, 1),
                    (0b11100001, 1)]);
}
