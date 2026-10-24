// Copyright 2025 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::os::raw::c_int;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

const M_MMAP_THRESHOLD: c_int = -3;
const TRIM_INTERVAL_MS: u64 = 5_000;

static TRIM_EPOCH: OnceLock<Instant> = OnceLock::new();
static LAST_TRIM_MS: AtomicU64 = AtomicU64::new(0);

unsafe extern "C" {
    fn malloc_trim(pad: usize) -> c_int;

    fn mallopt(param: c_int, value: c_int) -> c_int;
}

#[inline]
pub fn trim(pad: usize) {
    let elapsed = TRIM_EPOCH.get_or_init(Instant::now).elapsed().as_millis();
    let now = u64::try_from(elapsed).unwrap_or(u64::MAX);
    if !claim_trim(&LAST_TRIM_MS, now) {
        return;
    }

    // SAFETY: malloc_trim is thread-safe and does not invalidate live allocations.
    unsafe {
        malloc_trim(pad);
    }
}

fn claim_trim(last: &AtomicU64, now: u64) -> bool {
    let previous = last.load(Ordering::Relaxed);
    now.saturating_sub(previous) >= TRIM_INTERVAL_MS
        && last
            .compare_exchange(previous, now, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
}

/// Prevents glibc from hoarding memory via memory fragmentation.
#[inline]
pub fn limit_mmap_threshold(threshold: i32) {
    // SAFETY: mallopt accepts M_MMAP_THRESHOLD and does not invalidate allocations.
    unsafe {
        mallopt(M_MMAP_THRESHOLD, threshold as c_int);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routine_updates_defer_trimming_and_periodic_updates_still_reclaim() {
        let last = AtomicU64::new(0);
        for now in 0..TRIM_INTERVAL_MS {
            assert!(!claim_trim(&last, now));
        }
        assert!(claim_trim(&last, TRIM_INTERVAL_MS));
        for now in TRIM_INTERVAL_MS..2 * TRIM_INTERVAL_MS {
            assert!(!claim_trim(&last, now));
        }
        assert!(claim_trim(&last, 2 * TRIM_INTERVAL_MS));
        assert!(claim_trim(&last, 20 * TRIM_INTERVAL_MS));
        assert!(!claim_trim(&last, TRIM_INTERVAL_MS));
    }

    #[test]
    fn concurrent_updates_claim_only_one_trim() {
        let last = AtomicU64::new(0);
        let barrier = std::sync::Barrier::new(16);
        let count = std::thread::scope(|scope| {
            let threads = (0..16)
                .map(|_| {
                    scope.spawn(|| {
                        barrier.wait();
                        claim_trim(&last, TRIM_INTERVAL_MS)
                    })
                })
                .collect::<Vec<_>>();
            threads
                .into_iter()
                .map(|thread| thread.join().unwrap())
                .filter(|claimed| *claimed)
                .count()
        });
        assert_eq!(count, 1);
    }
}
