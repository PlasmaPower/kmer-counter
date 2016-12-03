use count::*;

#[test]
fn simple_count() {
    ::env_logger::init().ok();

    let input = b"AGGCT\nGGAC";
    let count_opts = Options {
        kmer_len: 3,
        kmer_bitmask: 0b111111,
    };

    let counts = count(input, &count_opts);
    assert_eq!(counts,
               vec![(0b001010, 1),
                    (0b101001, 1),
                    (0b100111, 1),
                    (0b011110, 1),
                    (0b111010, 1),
                    (0b101000, 1),
                    (0b100001, 1)]);
}
