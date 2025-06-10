//! RGB LED Calibration Tool
//!
//! This embedded Rust application runs on a MicroBit v2 to calibrate RGB LED timing
//! for creating white light using time-division multiplexing. The tool allows adjusting
//! frame rate and individual color brightness levels to find optimal values for
//! producing visually white light from an RGB LED without current-limiting resistors.
//!
//! # Hardware Setup
//! - RGB LED connected to pins P9 (red), P8 (green), P16 (blue)
//! - Potentiometer connected to P2 for analog input
//! - Uses MicroBit v2 buttons A and B for mode selection
//!
//! # Architecture
//! The application uses Embassy async framework with two concurrent tasks:
//! - RGB task: Handles time-division multiplexing of LED colors
//! - UI task: Processes user input from knob and buttons

#![no_std]
#![no_main]

mod knob;
mod rgb;
mod ui;
pub use knob::*;
pub use rgb::*;
pub use ui::*;

// Panic handler for embedded environment
use panic_rtt_target as _;
// RTT (Real-Time Transfer) for debug printing over probe
use rtt_target::{rprintln, rtt_init_print};

// Embassy async runtime for embedded systems
use embassy_executor::Spawner;
use embassy_futures::join;
// Synchronization primitives for sharing data between async tasks
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::Timer;
// MicroBit hardware abstraction layer
use microbit_bsp::{
    embassy_nrf::{
        bind_interrupts,
        gpio::{AnyPin, Level, Output, OutputDrive},
        saadc, // Successive Approximation ADC for analog input
    },
    Button, Microbit,
};
use num_traits::float::FloatCore;

/// Global shared state for RGB brightness levels [red, green, blue]
/// Protected by mutex for safe access between async tasks
pub static RGB_LEVELS: Mutex<ThreadModeRawMutex, [u32; 3]> = Mutex::new([0; 3]);

/// Global shared state for frame rate (frames per second)
/// Protected by mutex for safe access between async tasks
pub static FRAME_RATE: Mutex<ThreadModeRawMutex, u64> = Mutex::new(100);

/// Number of brightness levels per color (0-15, giving 16 total levels)
pub const LEVELS: u32 = 16;

/// Safely read the current RGB brightness levels from shared state
///
/// Returns: Array of [red, green, blue] brightness values (0-15)
async fn get_rgb_levels() -> [u32; 3] {
    let rgb_levels = RGB_LEVELS.lock().await;
    *rgb_levels
}

/// Safely modify the RGB brightness levels in shared state
///
/// # Arguments
/// * `setter` - Closure that modifies the RGB levels array
async fn set_rgb_levels<F>(setter: F)
where
    F: FnOnce(&mut [u32; 3]),
{
    let mut rgb_levels = RGB_LEVELS.lock().await;
    setter(&mut rgb_levels);
}

/// Safely read the current frame rate from shared state
///
/// Returns: Current frame rate in frames per second
async fn get_frame_rate() -> u64 {
    let frame_rate = FRAME_RATE.lock().await;
    *frame_rate
}

/// Safely modify the frame rate in shared state
///
/// # Arguments
/// * `new_rate` - New frame rate in frames per second
async fn set_frame_rate(new_rate: u64) {
    let mut frame_rate = FRAME_RATE.lock().await;
    *frame_rate = new_rate;
}

/// Main entry point for the RGB LED calibration application
///
/// Sets up hardware peripherals and launches concurrent RGB and UI tasks.
/// The function never returns (indicated by `!` return type).
#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    // Initialize RTT for debug output
    rtt_init_print!();
    // Get default MicroBit hardware configuration
    let board = Microbit::default();

    // Bind SAADC interrupt handler for ADC conversions
    bind_interrupts!(struct Irqs {
        SAADC => saadc::InterruptHandler;
    });

    // Configure GPIO pins for RGB LED control (active high, standard drive)
    let led_pin = |p| Output::new(p, Level::Low, OutputDrive::Standard);
    let red = led_pin(AnyPin::from(board.p9)); // Red LED on pin P9
    let green = led_pin(AnyPin::from(board.p8)); // Green LED on pin P8
    let blue = led_pin(AnyPin::from(board.p16)); // Blue LED on pin P16
                                                 // Create RGB controller with 100 fps initial frame rate
    let rgb: Rgb = Rgb::new([red, green, blue], 100);

    // Configure ADC for potentiometer reading with 14-bit resolution
    let mut saadc_config = saadc::Config::default();
    saadc_config.resolution = saadc::Resolution::_14BIT;
    let saadc = saadc::Saadc::new(
        board.saadc,
        Irqs,
        saadc_config,
        [saadc::ChannelConfig::single_ended(board.p2)], // Potentiometer on P2
    );
    // Create knob interface with calibrated ADC
    let knob = Knob::new(saadc).await;
    // Create UI handler with knob and button inputs
    let mut ui = Ui::new(knob, board.btn_a, board.btn_b);

    // Run RGB scanning and UI tasks concurrently - this never returns
    join::join(rgb.run(), ui.run()).await;

    // Should never reach here
    panic!("fell off end of main loop");
}
