// Simple timer using CPU Time Stamp Counter (RDTSC).
// Provides rdtsc read and busy-wait delays.
// Note: Accurate millisecond delays require a calibrated TSC frequency.

pub struct Timer;

impl Timer {
    pub fn init() -> Self {
        Timer
    }

    #[inline]
    pub fn rdtsc() -> u64 {
        let low: u32;
        let high: u32;
        unsafe {
            core::arch::asm!(
                "rdtsc",
                out("eax") low,
                out("edx") high,
                options(nomem, nostack, preserves_flags)
            );
        }
        ((high as u64) << 32) | (low as u64)
    }

    /// Busy-wait for approximately `cycles` TSC ticks.
    pub fn delay_cycles(&self, cycles: u64) {
        let start = Self::rdtsc();
        while Self::rdtsc().wrapping_sub(start) < cycles {
            core::hint::spin_loop();
        }
    }

    /// Delay for approximately `ms` milliseconds given tsc_hz frequency in Hz.
    pub fn delay_ms(&self, ms: u64, tsc_hz: u64) {
        if tsc_hz == 0 { return; }
        let cycles = ms.saturating_mul(tsc_hz / 1000);
        self.delay_cycles(cycles);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rdtsc_reads() {
        let _ = Timer::rdtsc();
    }
}
