//! Deterministic allocation contracts for public hot paths.

#![cfg(not(target_arch = "wasm32"))]

use std::fmt::{self, Write as _};
use std::hint::black_box;

use allocation_counter::{measure, AllocationInfo};
use zlid::{
    advanced::EntropySource,
    wire::{pack_ordered, unpack_ordered},
    OrderedGenerator, Profile, ZLID,
};

const CANONICAL: &str = "01K2R7KFWE5807000000000001";
const FRIENDLY: &str = "01K2R7KF-WE580_7000000000001";
const KEY: &[u8] = b"0123456789abcdef0123456789abcdef";
const TWEAK: &[u8] = b"allocation-contract";

#[derive(Default)]
struct FixedEntropy(u8);

struct StackOutput {
    bytes: [u8; 64],
    length: usize,
}

impl StackOutput {
    fn new() -> Self {
        Self {
            bytes: [0; 64],
            length: 0,
        }
    }

    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.length]).expect("formatter emits UTF-8")
    }
}

impl fmt::Write for StackOutput {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.length + text.len();
        if end > self.bytes.len() {
            return Err(fmt::Error);
        }
        self.bytes[self.length..end].copy_from_slice(text.as_bytes());
        self.length = end;
        Ok(())
    }
}

impl EntropySource for FixedEntropy {
    fn fill_bytes(&mut self, output: &mut [u8]) -> zlid::Result<()> {
        output.fill(self.0);
        self.0 = self.0.wrapping_add(1);
        Ok(())
    }
}

fn ordered() -> ZLID {
    ZLID::from_array(
        pack_ordered(
            Profile::Default,
            1_726_000_000_000,
            17,
            3,
            0x12_3456_789a_bcde,
            1,
        )
        .expect("valid ordered fixture"),
    )
}

fn assert_exact(label: &str, expected: u64, info: AllocationInfo) {
    assert_eq!(info.count_total, expected, "{label}: {info:?}");
    assert_eq!(info.count_current, 0, "{label} leaked: {info:?}");
    assert_eq!(info.bytes_current, 0, "{label} leaked: {info:?}");
}

fn assert_at_most(label: &str, maximum: u64, info: AllocationInfo) {
    assert!(
        info.count_total <= maximum,
        "{label} allocated more than {maximum} times: {info:?}"
    );
    assert_eq!(info.count_current, 0, "{label} leaked: {info:?}");
    assert_eq!(info.bytes_current, 0, "{label} leaked: {info:?}");
}

#[test]
fn parsing_and_binary_hot_paths_do_not_allocate() {
    assert_exact(
        "parse",
        0,
        measure(|| {
            black_box(ZLID::parse(black_box(CANONICAL)).unwrap());
        }),
    );
    assert_exact(
        "friendly parse",
        0,
        measure(|| {
            black_box(ZLID::parse(black_box(FRIENDLY)).unwrap());
        }),
    );

    let bytes = ordered().bytes();
    assert_exact(
        "pack_ordered",
        0,
        measure(|| {
            black_box(
                pack_ordered(
                    Profile::Default,
                    black_box(7),
                    black_box(3),
                    black_box(2),
                    black_box(1),
                    black_box(1),
                )
                .unwrap(),
            );
        }),
    );
    assert_exact(
        "unpack_ordered",
        0,
        measure(|| {
            black_box(unpack_ordered(black_box(&bytes)).unwrap());
        }),
    );
    assert_exact(
        "partition_bytes",
        0,
        measure(|| {
            black_box(ZLID::partition_bytes(black_box(b"tenant-17"), None).unwrap());
        }),
    );
}

#[test]
fn generation_hot_paths_do_not_allocate_after_warmup() {
    let mut entropy = FixedEntropy::default();
    assert_exact(
        "random_with",
        0,
        measure(|| {
            black_box(ZLID::random_with(&mut entropy).unwrap());
        }),
    );

    let mut generator = OrderedGenerator::with_sources(
        Profile::Default,
        17,
        || 1_726_000_000_000,
        FixedEntropy::default(),
    );
    black_box(generator.next().unwrap());
    assert_exact(
        "warmed ordered generation",
        0,
        measure(|| {
            black_box(generator.next().unwrap());
        }),
    );
}

#[test]
fn alias_round_trips_do_not_allocate() {
    let source = ordered();
    let alias = source.alias(KEY, TWEAK).unwrap();
    let long_key = [0x42; 131];
    let maximum_tweak = [0x24; 65_535];
    let long_alias = source.alias(&long_key, &maximum_tweak).unwrap();

    assert_exact(
        "alias",
        0,
        measure(|| {
            black_box(source.alias(black_box(KEY), black_box(TWEAK)).unwrap());
        }),
    );
    assert_exact(
        "unalias",
        0,
        measure(|| {
            black_box(alias.unalias(black_box(KEY), black_box(TWEAK)).unwrap());
        }),
    );
    assert_exact(
        "alias with long key and maximum tweak",
        0,
        measure(|| {
            black_box(
                source
                    .alias(black_box(&long_key), black_box(&maximum_tweak))
                    .unwrap(),
            );
        }),
    );
    assert_exact(
        "unalias with long key and maximum tweak",
        0,
        measure(|| {
            black_box(
                long_alias
                    .unalias(black_box(&long_key), black_box(&maximum_tweak))
                    .unwrap(),
            );
        }),
    );
}

#[test]
fn owned_text_and_hex_results_allocate_once() {
    let id = ordered();
    assert_exact(
        "text",
        1,
        measure(|| {
            black_box(id.text());
        }),
    );
    assert_exact(
        "ZLID::bytes_hex",
        1,
        measure(|| {
            black_box(id.bytes_hex());
        }),
    );
}

#[test]
fn display_and_debug_format_without_intermediate_allocations() {
    let id = ordered();
    let canonical = id.text();
    let debug = format!("ZLID(\"{canonical}\")");

    let mut display_output = StackOutput::new();
    write!(&mut display_output, "{id}").unwrap();
    assert_eq!(display_output.as_str(), canonical);
    let mut debug_output = StackOutput::new();
    write!(&mut debug_output, "{id:?}").unwrap();
    assert_eq!(debug_output.as_str(), debug);

    assert_exact(
        "Display",
        0,
        measure(|| {
            let mut output = StackOutput::new();
            write!(&mut output, "{}", black_box(id)).unwrap();
            black_box(output);
        }),
    );
    assert_exact(
        "Debug",
        0,
        measure(|| {
            let mut output = StackOutput::new();
            write!(&mut output, "{:?}", black_box(id)).unwrap();
            black_box(output);
        }),
    );
}

#[test]
fn inspection_stays_within_its_owned_output_budget() {
    let ordered = ordered();
    let alias = ordered.alias(KEY, TWEAK).unwrap();
    let random = ZLID::from_array([0x15; 16]);
    let opaque = ZLID::from_array([0x1a; 16]);

    for (label, id) in [("ordered", ordered), ("random", random), ("alias", alias)] {
        assert_at_most(
            label,
            3,
            measure(|| {
                black_box(id.inspect());
            }),
        );
    }
    for (label, id) in [("sentinel", ZLID::NIL), ("opaque", opaque)] {
        assert_at_most(
            label,
            2,
            measure(|| {
                black_box(id.inspect());
            }),
        );
    }
}
