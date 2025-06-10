//! RGB LED Time-Division Multiplexing Controller
//!
//! This module implements time-division multiplexing for RGB LEDs to create
//! mixed colors including white. Since only one LED can be on at a time due to
//! hardware constraints (no current-limiting resistors), rapid switching between
//! colors creates the illusion of mixed colors through persistence of vision.

use crate::*;

/// Type alias for the three RGB LED output pins [red, green, blue]
type RgbPins = [Output<'static, AnyPin>; 3];

/// RGB LED controller using time-division multiplexing
/// 
/// Controls three LED pins with precise timing to create mixed colors.
/// Each color is displayed for a time proportional to its brightness level.
pub struct Rgb {
    /// GPIO output pins for [red, green, blue] LEDs
    rgb: RgbPins,
    /// Shadow copy of brightness levels to minimize mutex lock contention
    /// Values range from 0 (off) to LEVELS-1 (full brightness)
    levels: [u32; 3],
    /// Time in microseconds for each brightness tick
    /// Calculated from frame rate: 1_000_000 / (3 * frame_rate * LEVELS)
    tick_time: u64,
}

impl Rgb {
    /// Calculate tick time in microseconds from frame rate
    /// 
    /// Frame rate determines how many complete RGB scans occur per second.
    /// Each frame has 3 colors × LEVELS brightness steps, so:
    /// tick_time = 1_000_000 μs/sec ÷ (3 colors × frame_rate × LEVELS)
    /// 
    /// # Arguments
    /// * `frame_rate` - Target frames per second
    /// 
    /// # Returns
    /// Microseconds per brightness tick
    fn frame_tick_time(frame_rate: u64) -> u64 {
        1_000_000 / (3 * frame_rate * LEVELS as u64)
    }

    /// Create a new RGB controller with specified pins and frame rate
    /// 
    /// # Arguments
    /// * `rgb` - Array of GPIO output pins [red, green, blue]
    /// * `frame_rate` - Target refresh rate in frames per second
    /// 
    /// # Returns
    /// New RGB controller instance
    pub fn new(rgb: RgbPins, frame_rate: u64) -> Self {
        let tick_time = Self::frame_tick_time(frame_rate);
        Self {
            rgb,
            levels: [0; 3],  // Start with all LEDs off
            tick_time,
        }
    }

    /// Execute one time slice for a single LED color
    /// 
    /// This implements pulse-width modulation by turning the LED on for a time
    /// proportional to its brightness level, then off for the remaining time.
    /// Total time per step is always the same to maintain consistent frame rate.
    /// 
    /// # Arguments
    /// * `led` - LED index (0=red, 1=green, 2=blue)
    async fn step(&mut self, led: usize) {
        let level = self.levels[led];
        
        // Turn LED on for time proportional to brightness level
        if level > 0 {
            self.rgb[led].set_high();
            let on_time = level as u64 * self.tick_time;
            Timer::after_micros(on_time).await;
            self.rgb[led].set_low();
        }
        
        // Turn LED off for remaining time to complete the time slice
        let off_level = LEVELS - level;
        if off_level > 0 {
            let off_time = off_level as u64 * self.tick_time;
            Timer::after_micros(off_time).await;
        }
    }

    /// Main RGB scanning loop - runs forever
    /// 
    /// Continuously cycles through red, green, and blue LEDs, displaying each
    /// for a time proportional to its brightness setting. Updates brightness
    /// levels and frame rate from shared state each frame to maintain
    /// consistent timing.
    /// 
    /// The loop never returns, indicated by the `!` return type.
    pub async fn run(mut self) -> ! {
        loop {
            // Get latest brightness levels from UI
            self.levels = get_rgb_levels().await;
            
            // Get current frame rate and update tick time if changed
            let current_frame_rate = get_frame_rate().await;
            let expected_tick_time = Self::frame_tick_time(current_frame_rate);
            if self.tick_time != expected_tick_time {
                self.tick_time = expected_tick_time;
            }

            // Scan through each color: red (0), green (1), blue (2)
            for led in 0..3 {
                self.step(led).await;
            }
        }
    }
}
