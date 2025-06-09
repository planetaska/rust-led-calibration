//! Potentiometer (Knob) Interface
//!
//! Provides calibrated analog input from a potentiometer connected to the MicroBit's
//! ADC. Converts raw ADC readings to discrete brightness levels (0 to LEVELS-1)
//! with proper scaling and clamping.

use crate::*;

/// Type alias for the SAADC (Successive Approximation ADC) with 1 channel
pub type Adc = saadc::Saadc<'static, 1>;

/// Potentiometer interface for user input
/// 
/// Wraps the ADC to provide calibrated readings from a potentiometer.
/// The potentiometer should be wired with:
/// - Pin 1 to GND
/// - Pin 2 to P2 (ADC input) 
/// - Pin 3 to +3.3V
pub struct Knob(Adc);

impl Knob {
    /// Create a new knob interface with calibrated ADC
    /// 
    /// Performs ADC calibration to ensure accurate readings across
    /// the full voltage range.
    /// 
    /// # Arguments
    /// * `adc` - Configured SAADC instance
    /// 
    /// # Returns
    /// Calibrated knob interface ready for measurements
    pub async fn new(adc: Adc) -> Self {
        // Calibrate ADC for accurate voltage measurements
        adc.calibrate().await;
        Self(adc)
    }

    /// Read potentiometer position and convert to brightness level
    /// 
    /// Performs ADC sampling and converts the raw reading to a discrete
    /// brightness level from 0 to LEVELS-1 (0 to 15).
    /// 
    /// The conversion applies scaling and offset to map the ADC range
    /// to brightness levels with some margin for mechanical tolerances.
    /// 
    /// # Returns
    /// Brightness level (0 = minimum, LEVELS-1 = maximum)
    pub async fn measure(&mut self) -> u32 {
        let mut buf = [0];
        // Sample ADC (blocks until conversion complete)
        self.0.sample(&mut buf).await;
        
        // Clamp raw reading to positive 15-bit range (14-bit ADC + sign)
        let raw = buf[0].clamp(0, 0x7fff) as u16;
        
        // Scale to 0.0-1.0 range (division factor tuned for hardware)
        let scaled = raw as f32 / 10_000.0;
        
        // Map to brightness levels with offset for better range coverage
        // Formula provides some margin at both ends of knob travel
        let result = ((LEVELS + 2) as f32 * scaled - 2.0)
            .clamp(0.0, (LEVELS - 1) as f32)  // Ensure valid range
            .floor();  // Convert to integer level
            
        result as u32
    }
}
