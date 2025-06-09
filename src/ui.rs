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
    /// Button A input (currently unused - to be implemented)
    _button_a: Button,
    /// Button B input (currently unused - to be implemented)
    _button_b: Button,
    /// Current UI state (brightness levels and frame rate)
    state: UiState,
}

impl Ui {
    /// Create a new UI controller with specified hardware interfaces
    /// 
    /// # Arguments
    /// * `knob` - Calibrated potentiometer interface
    /// * `_button_a` - MicroBit button A (to be implemented)
    /// * `_button_b` - MicroBit button B (to be implemented)
    /// 
    /// # Returns
    /// New UI controller with default initial state
    pub fn new(knob: Knob, _button_a: Button, _button_b: Button) -> Self {
        Self {
            knob,
            _button_a,
            _button_b,
            state: UiState::default(),
        }
    }

    /// Main UI processing loop - runs forever
    /// 
    /// Currently implements basic blue LED control via knob.
    /// TODO: Add button handling for frame rate and red/green control.
    /// 
    /// The loop:
    /// 1. Reads knob position
    /// 2. Updates blue brightness level if changed
    /// 3. Updates shared RGB state
    /// 4. Displays current state
    /// 5. Waits 50ms before next reading
    pub async fn run(&mut self) -> ! {
        // Initialize blue level from knob position
        self.state.levels[2] = self.knob.measure().await;
        // Update shared state with initial values
        set_rgb_levels(|rgb| {
            *rgb = self.state.levels;
        })
        .await;
        // Show initial state
        self.state.show();
        
        loop {
            // Read current knob position (0 to LEVELS-1)
            let level = self.knob.measure().await;
            
            // Update blue level if knob moved
            if level != self.state.levels[2] {
                self.state.levels[2] = level;
                self.state.show(); // Display updated state
                
                // Update shared RGB state for the RGB task
                set_rgb_levels(|rgb| {
                    *rgb = self.state.levels;
                })
                .await;
            }
            
            // Poll at 20Hz (every 50ms) to balance responsiveness and CPU usage
            Timer::after_millis(50).await;
        }
    }
}
