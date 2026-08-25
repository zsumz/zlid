//! Stable, dependency-free microbenchmarks for public ZLID hot paths.

use std::hint::black_box;
use std::time::{Duration, Instant};

use zlid::advanced::{EntropySource, SystemEntropy};
use zlid::wire::{pack_ordered, unpack_ordered};
use zlid::{OrderedGenerator, Profile, ZLID};

const SAMPLES: usize = 21;
const PARALLEL_SAMPLES: usize = 7;
const MIN_SAMPLE: Duration = Duration::from_millis(25);
const WARMUP: Duration = Duration::from_secs(2);
const VARIANTS: usize = 16;
const KEY: &[u8] = b"0123456789abcdef0123456789abcdef";
const SHORT_TWEAK: &[u8] = b"hot-path";

#[derive(Default)]
struct FixedEntropy(u8);

impl EntropySource for FixedEntropy {
    fn fill_bytes(&mut self, output: &mut [u8]) -> zlid::Result<()> {
        output.fill(self.0);
        self.0 = self.0.wrapping_add(1);
        Ok(())
    }
}

fn fixtures() -> [ZLID; VARIANTS] {
    std::array::from_fn(|index| {
        ZLID::from_array(
            pack_ordered(
                Profile::Default,
                1_726_000_000_000 + index as u64,
                index as u8,
                index as u32,
                0x12_3456_789a_bcde ^ index as u64,
                1,
            )
            .expect("valid benchmark fixture"),
        )
    })
}

fn run_batch(iterations: u64, nonce: &mut u64, operation: &mut impl FnMut(u64)) -> Duration {
    let started = Instant::now();
    for _ in 0..iterations {
        operation(black_box(*nonce));
        *nonce = nonce.wrapping_add(1);
    }
    started.elapsed()
}

fn calibrated_iterations(nonce: &mut u64, operation: &mut impl FnMut(u64)) -> u64 {
    let mut iterations = 1u64;
    loop {
        let elapsed = run_batch(iterations, nonce, operation);
        if elapsed >= MIN_SAMPLE {
            return iterations;
        }
        let multiplier = (MIN_SAMPLE.as_nanos() / elapsed.as_nanos().max(1)).clamp(2, 256);
        iterations = iterations.saturating_mul(multiplier as u64);
    }
}

fn report(lane: &str, name: &str, mut operation: impl FnMut(u64)) {
    let mut nonce = 0;
    let iterations = calibrated_iterations(&mut nonce, &mut operation);
    let warmup_started = Instant::now();
    while warmup_started.elapsed() < WARMUP {
        run_batch(iterations, &mut nonce, &mut operation);
    }

    let mut samples = [0.0; SAMPLES];
    for sample in &mut samples {
        let elapsed = run_batch(iterations, &mut nonce, &mut operation);
        *sample = elapsed.as_nanos() as f64 / iterations as f64;
    }
    samples.sort_by(f64::total_cmp);
    let median = samples[SAMPLES / 2];
    let mut deviations = samples.map(|sample| (sample - median).abs());
    deviations.sort_by(f64::total_cmp);
    let mad = deviations[SAMPLES / 2];
    println!("{lane:13} {name:23} {median:>9.2} ns/op  MAD {mad:>7.2}");
}

fn deterministic_lanes() {
    let ids = fixtures();
    let texts = ids.each_ref().map(|id| id.text());
    let friendly = texts
        .each_ref()
        .map(|text| format!("{}-{}_{}", &text[..8], &text[8..17], &text[17..]));
    let aliases = ids.each_ref().map(|id| id.alias(KEY, SHORT_TWEAK).unwrap());
    let short_inputs: [[u8; 11]; VARIANTS] = std::array::from_fn(|index| {
        let mut input = [0x5a; 11];
        input[0] = index as u8;
        input
    });
    let long_inputs: [[u8; 1024]; VARIANTS] = std::array::from_fn(|index| {
        let mut input = [0x5a; 1024];
        input[0] = index as u8;
        input
    });
    let long_tweak = [0x6b; 1024];

    report("deterministic", "pack ordered", |nonce| {
        black_box(
            pack_ordered(
                Profile::Default,
                black_box(nonce & ((1 << 48) - 1)),
                black_box(nonce as u8),
                black_box(nonce as u32 & 0x0fff),
                black_box(nonce & ((1 << 56) - 1)),
                black_box(1),
            )
            .unwrap(),
        );
    });
    report("deterministic", "unpack ordered", |nonce| {
        black_box(unpack_ordered(black_box(ids[nonce as usize % VARIANTS].as_bytes())).unwrap());
    });
    report("deterministic", "encode text", |nonce| {
        black_box(ids[nonce as usize % VARIANTS].text());
    });
    report("deterministic", "parse canonical", |nonce| {
        black_box(ZLID::parse(black_box(&texts[nonce as usize % VARIANTS])).unwrap());
    });
    report("deterministic", "parse friendly", |nonce| {
        black_box(ZLID::parse(black_box(&friendly[nonce as usize % VARIANTS])).unwrap());
    });
    report("deterministic", "partition 11 bytes", |nonce| {
        black_box(
            ZLID::partition_bytes(black_box(&short_inputs[nonce as usize % VARIANTS]), None)
                .unwrap(),
        );
    });
    report("deterministic", "partition 1 KiB", |nonce| {
        black_box(
            ZLID::partition_bytes(black_box(&long_inputs[nonce as usize % VARIANTS]), None)
                .unwrap(),
        );
    });
    report("deterministic", "alias short tweak", |nonce| {
        black_box(
            ids[nonce as usize % VARIANTS]
                .alias(black_box(KEY), black_box(SHORT_TWEAK))
                .unwrap(),
        );
    });
    report("deterministic", "alias 1 KiB tweak", |nonce| {
        black_box(
            ids[nonce as usize % VARIANTS]
                .alias(black_box(KEY), black_box(&long_tweak))
                .unwrap(),
        );
    });
    report("deterministic", "unalias short tweak", |nonce| {
        black_box(
            aliases[nonce as usize % VARIANTS]
                .unalias(black_box(KEY), black_box(SHORT_TWEAK))
                .unwrap(),
        );
    });

    let mut entropy = FixedEntropy::default();
    report("deterministic", "random injected", |_| {
        black_box(ZLID::random_with(&mut entropy).unwrap());
    });
    let mut generator = OrderedGenerator::with_sources(
        Profile::Default,
        17,
        || 1_726_000_000_000,
        FixedEntropy::default(),
    );
    black_box(generator.next().unwrap());
    report("deterministic", "ordered injected", |_| {
        black_box(generator.next().unwrap());
    });
    report("deterministic", "inspect ordered", |nonce| {
        black_box(ids[nonce as usize % VARIANTS].inspect());
    });
}

fn advisory_lanes() {
    let mut system_entropy = SystemEntropy;
    report("advisory", "random system", |_| {
        black_box(ZLID::random_with(&mut system_entropy).unwrap());
    });
    report("advisory", "ordered shared", |nonce| {
        black_box(ZLID::next_with_partition(nonce as u8 & 3).unwrap());
    });
    for threads in [1, 2, 4, 8] {
        parallel_generation(ParallelLane::ExplicitDistinct, threads);
        parallel_generation(ParallelLane::SharedSame, threads);
        parallel_generation(ParallelLane::SharedDistinct, threads);
    }
}

#[derive(Clone, Copy)]
enum ParallelLane {
    ExplicitDistinct,
    SharedSame,
    SharedDistinct,
}

impl ParallelLane {
    fn name(self) -> &'static str {
        match self {
            Self::ExplicitDistinct => "explicit distinct",
            Self::SharedSame => "shared same",
            Self::SharedDistinct => "shared distinct",
        }
    }

    fn partition(self, worker: u8) -> u8 {
        match self {
            Self::SharedSame => 0,
            Self::ExplicitDistinct | Self::SharedDistinct => worker,
        }
    }
}

fn parallel_generation(lane: ParallelLane, threads: u64) {
    const PER_THREAD: u64 = 100_000;
    let mut samples = [0.0; PARALLEL_SAMPLES];
    for sample in &mut samples {
        let started = Instant::now();
        std::thread::scope(|scope| {
            for worker in 0..threads as u8 {
                let partition = lane.partition(worker);
                scope.spawn(move || {
                    let mut generator = OrderedGenerator::new(Profile::Default, partition);
                    for _ in 0..PER_THREAD {
                        let id = match lane {
                            ParallelLane::ExplicitDistinct => generator.next().unwrap(),
                            ParallelLane::SharedSame | ParallelLane::SharedDistinct => {
                                ZLID::next_with_partition(partition).unwrap()
                            }
                        };
                        black_box(id);
                    }
                });
            }
        });
        *sample = (threads * PER_THREAD) as f64 / started.elapsed().as_secs_f64();
    }
    samples.sort_by(f64::total_cmp);
    let median = samples[PARALLEL_SAMPLES / 2];
    let mut deviations = samples.map(|sample| (sample - median).abs());
    deviations.sort_by(f64::total_cmp);
    let mad = deviations[PARALLEL_SAMPLES / 2];
    let name = format!("{} {threads}t", lane.name());
    println!(
        "{:13} {name:23} {median:>9.0} ops/s  MAD {mad:>7.0}",
        "advisory"
    );
}

fn smoke() {
    let id = fixtures()[0];
    let text = id.text();
    let alias = id.alias(KEY, SHORT_TWEAK).unwrap();
    black_box(ZLID::parse(&text).unwrap());
    black_box(alias.unalias(KEY, SHORT_TWEAK).unwrap());
    black_box(unpack_ordered(id.as_bytes()).unwrap());
    black_box(ZLID::partition_bytes(b"smoke", None).unwrap());
    let mut entropy = FixedEntropy::default();
    black_box(ZLID::random_with(&mut entropy).unwrap());
}

fn main() {
    let measure = std::env::args()
        .skip(1)
        .any(|argument| argument == "--measure");
    if !measure {
        smoke();
        println!("benchmark smoke passed; add -- --measure for timings");
        return;
    }

    println!(
        "ZLID hot paths on {}-{}; median/MAD timings are advisory",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    deterministic_lanes();
    advisory_lanes();
}
