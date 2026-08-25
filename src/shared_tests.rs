use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::time::Duration;

use super::SharedOrderedGenerator;
use crate::{EntropySource, Error, Inspection};

struct SignalingEntropy {
    signal: Sender<usize>,
    fail: bool,
}

impl EntropySource for SignalingEntropy {
    fn fill_bytes(&mut self, out: &mut [u8]) -> crate::Result<()> {
        self.signal.send(out.len()).expect("signal receiver");
        if self.fail {
            return Err(Error::EntropyUnavailable(
                "injected entropy failure".to_string(),
            ));
        }
        out.fill(0);
        Ok(())
    }
}

#[test]
fn entropy_is_drawn_before_the_shared_state_lock() {
    let generator = Arc::new(SharedOrderedGenerator::new());
    let guard = generator.0.lock().expect("shared state lock");
    let (entropy_tx, entropy_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    let thread_generator = Arc::clone(&generator);

    let handle = std::thread::spawn(move || {
        let mut entropy = SignalingEntropy {
            signal: entropy_tx,
            fail: false,
        };
        result_tx
            .send(thread_generator.next_with_entropy(17, &mut entropy))
            .expect("result receiver");
    });

    assert_eq!(entropy_rx.recv_timeout(Duration::from_secs(1)).unwrap(), 7);
    assert!(result_rx.try_recv().is_err());
    drop(guard);

    let id = result_rx
        .recv_timeout(Duration::from_secs(1))
        .unwrap()
        .unwrap();
    handle.join().expect("generation thread");
    let Inspection::Ordered { partition, .. } = id.inspect() else {
        panic!("shared generator returned a non-ordered ID");
    };
    assert_eq!(partition, 17);
}

#[test]
fn entropy_failure_neither_waits_for_the_lock_nor_initializes_state() {
    let generator = Arc::new(SharedOrderedGenerator::new());
    let guard = generator.0.lock().expect("shared state lock");
    let (entropy_tx, entropy_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    let thread_generator = Arc::clone(&generator);

    let handle = std::thread::spawn(move || {
        let mut entropy = SignalingEntropy {
            signal: entropy_tx,
            fail: true,
        };
        result_tx
            .send(thread_generator.next_with_entropy(29, &mut entropy))
            .expect("result receiver");
    });

    assert_eq!(entropy_rx.recv_timeout(Duration::from_secs(1)).unwrap(), 7);
    assert!(matches!(
        result_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
        Err(Error::EntropyUnavailable(_))
    ));
    drop(guard);
    handle.join().expect("generation thread");

    let (signal, _receiver) = mpsc::channel();
    let mut entropy = SignalingEntropy {
        signal,
        fail: false,
    };
    let id = generator.next_with_entropy(29, &mut entropy).unwrap();
    let Inspection::Ordered {
        partition,
        sequence,
        ..
    } = id.inspect()
    else {
        panic!("shared generator returned a non-ordered ID");
    };
    assert_eq!((partition, sequence), (29, 0));
}

#[test]
fn invalid_predrawn_tail_does_not_initialize_stream() {
    let generator = SharedOrderedGenerator::new();
    let mut core = generator.0.lock().expect("shared state lock");

    assert!(matches!(
        core.next_with_random_tail(Some(43), u64::MAX),
        Err(Error::FieldOutOfRange { .. })
    ));
    let id = core.next_with_random_tail(Some(43), 0).unwrap();
    let Inspection::Ordered {
        partition,
        sequence,
        ..
    } = id.inspect()
    else {
        panic!("shared core returned a non-ordered ID");
    };
    assert_eq!((partition, sequence), (43, 0));
}
