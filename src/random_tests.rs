use std::cell::Cell;
use std::rc::Rc;

use super::{random_value, EntropySource};

struct RecordingEntropy {
    requested: Rc<Cell<usize>>,
    byte: u8,
}

impl EntropySource for RecordingEntropy {
    fn fill_bytes(&mut self, out: &mut [u8]) -> crate::Result<()> {
        self.requested.set(out.len());
        out.fill(self.byte);
        Ok(())
    }
}

#[test]
fn ordered_profiles_request_only_their_seven_entropy_bytes() {
    for (bits, expected) in [(52, 0x000A_AAAA_AAAA_AAAA), (56, 0x00AA_AAAA_AAAA_AAAA)] {
        let requested = Rc::new(Cell::new(usize::MAX));
        let mut entropy = RecordingEntropy {
            requested: Rc::clone(&requested),
            byte: 0xaa,
        };

        assert_eq!(random_value(&mut entropy, bits).unwrap(), expected);
        assert_eq!(requested.get(), 7);
    }
}

#[test]
fn stack_buffer_preserves_big_endian_masking_for_every_width() {
    for bits in 1..=64 {
        let requested = Rc::new(Cell::new(usize::MAX));
        let mut entropy = RecordingEntropy {
            requested: Rc::clone(&requested),
            byte: 0xff,
        };
        let expected = if bits == 64 {
            u64::MAX
        } else {
            (1u64 << bits) - 1
        };

        assert_eq!(random_value(&mut entropy, bits).unwrap(), expected);
        assert_eq!(requested.get(), usize::from(bits.div_ceil(8)));
    }
}
