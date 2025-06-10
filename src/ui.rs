//! User Interface Controller
//!
//! Handles user input from potentiometer knob and buttons to control RGB LED
//! brightness levels and frame rate. Provides real-time feedback via RTT debug output.

use crate::*;

/// Internal state for the user interface
/// 
/// Tracks current brightness levels and frame rate settings that are
/// controlled by knob position and button combinations.
struct UiState {
    /// RGB brightness levels [red, green, blue] (0 to LEVELS-1)
    levels: [u32; 3],
    /// Current frame rate in frames per second
    frame_rate: u64,
}

impl UiState {
    /// Display current RGB levels and frame rate via RTT debug output
    /// 
    /// Prints the current state to help users see the effect of their adjustments.
    /// Output format:
    /// ```
    /// red: 15
    /// green: 12  
    /// blue: 8
    /// frame rate: 100
    /// ```
    fn show(&self) {
        let names = ["red", "green", "blue"];
        rprintln!(); // Blank line for readability
        // Print each color level
        for (name, level) in names.iter().zip(self.levels.iter()) {
            rprintln!("{}: {}", name, level);
        }
        rprintln!("frame rate: {}", self.frame_rate);
    }
}

impl Default for UiState {
    /// Create initial UI state with sensible defaults
    /// 
    /// Starts with all colors at maximum brightness (LEVELS-1 = 15)
    /// and a moderate frame rate of 100 fps.
    fn default() -> Self {
        Self {
            // Start with all colors at max brightness for easy calibration
            levels: [LEVELS - 1, LEVELS - 1, LEVELS - 1],
            frame_rate: 100, // 100 fps default frame rate
        }
    }
}

/// User interface controller for RGB calibration
/// 
/// Manages knob input and button states to control which parameter
/// the knob adjusts. Button combinations determine the control mode:
/// - No buttons: Frame rate control (10-160 fps in steps of 10)
/// - A button: Blue brightness control (0-15)
/// - B button: Green brightness control (0-15)  
/// - A+B buttons: Red brightness control (0-15)
pub struct Ui {
    /// Potentiometer interface for analog input
    knob: Knob,
    /// Button A input for mode selection
    button_a: Button,
    /// Button B input for mode selection
    button_b: Button,
    /// Current UI state (brightness levels and frame rate)
    state: UiState,
}

impl Ui {
    /// Create a new UI controller with specified hardware interfaces
    /// 
    /// # Arguments
    /// * `knob` - Calibrated potentiometer interface
    /// * `button_a` - MicroBit button A for mode selection
    /// * `button_b` - MicroBit button B for mode selection
    /// 
    /// # Returns
    /// New UI controller with default initial state
    pub fn new(knob: Knob, button_a: Button, button_b: Button) -> Self {
        Self {
            knob,
            button_a,
            button_b,
            state: UiState::default(),
        }
    }

    /// Convert knob level (0-15) to frame rate (10-160 fps in steps of 10)
    /// 
    /// Maps the 16 knob positions to frame rates from 10 to 160 fps.
    /// Each step increases the frame rate by 10 fps.
    /// 
    /// # Arguments
    /// * `level` - Knob position (0 to LEVELS-1)
    /// 
    /// # Returns
    /// Frame rate in fps (10, 20, 30, ..., 160)
    fn level_to_frame_rate(level: u32) -> u64 {
        (level as u64 + 1) * 10
    }

    /// Main UI processing loop - runs forever
    /// 
    /// Handles knob input based on button state:
    /// - No buttons: Frame rate control (10-160 fps in steps of 10)
    /// - A button: Blue brightness control (0-15)
    /// - B button: Green brightness control (0-15)  
    /// - A+B buttons: Red brightness control (0-15)
    /// 
    /// The loop:
    /// 1. Reads button states
    /// 2. Reads knob position
    /// 3. Updates appropriate parameter based on button combination
    /// 4. Updates shared state if values changed
    /// 5. Displays current state
    /// 6. Waits 50ms before next reading
    pub async fn run(&mut self) -> ! {
        // Initialize state from current knob position
        let initial_level = self.knob.measure().await;
        self.state.frame_rate = Self::level_to_frame_rate(initial_level);
        
        // Initialize shared state
        set_rgb_levels(|rgb| {
            *rgb = self.state.levels;
        })
        .await;
        set_frame_rate(self.state.frame_rate).await;
        
        // Show initial state
        self.state.show();
        
        loop {
            // Read button states
            let button_a_pressed = self.button_a.is_low();
            let button_b_pressed = self.button_b.is_low();
            
            // Read current knob position (0 to LEVELS-1)
            let level = self.knob.measure().await;
            
            // Determine control mode and update appropriate parameter
            let mut state_changed = false;
            
            match (button_a_pressed, button_b_pressed) {
                (false, false) => {
                    // No buttons: Frame rate control
                    let new_frame_rate = Self::level_to_frame_rate(level);
                    if new_frame_rate != self.state.frame_rate {
                        self.state.frame_rate = new_frame_rate;
                        set_frame_rate(self.state.frame_rate).await;
                        state_changed = true;
                    }
                }
                (true, false) => {
                    // A button: Blue brightness control
                    if level != self.state.levels[2] {
                        self.state.levels[2] = level;
                        state_changed = true;
                    }
                }
                (false, true) => {
                    // B button: Green brightness control
                    if level != self.state.levels[1] {
                        self.state.levels[1] = level;
                        state_changed = true;
                    }
                }
                (true, true) => {
                    // A+B buttons: Red brightness control
                    if level != self.state.levels[0] {
                        self.state.levels[0] = level;
                        state_changed = true;
                    }
                }
            }
            
            // Update shared RGB state if brightness levels changed
            if state_changed {
                if button_a_pressed || button_b_pressed {
                    set_rgb_levels(|rgb| {
                        *rgb = self.state.levels;
                    })
                    .await;
                }
                self.state.show(); // Display updated state
            }
            
            // Poll at 20Hz (every 50ms) to balance responsiveness and CPU usage
            Timer::after_millis(50).await;
        }
    }
}
