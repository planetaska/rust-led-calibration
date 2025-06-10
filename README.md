# rgbcal: RGB LED calibration tool
Bart Massey 2024

This tool is designed to find out a decent frame rate and maximum RGB component values to produce a white-looking RGB of reasonable brightness.

## Author

Chia-Wei Hsu (chiawei@pdx.edu)

## Calibration Results

After testing with the MicroBit v2 hardware setup, the following values produce white light:

**RGB brightness levels for producing white light:**

* Red: 13/15
* Green: 6/15
* Blue: 10/15

**Minimum frame rate:**

* 60 fps (frames per second) to eliminate visible flicker

## Development Process

### What I Did

I started by reading through the provided documentation and code to fully understand the RGB LED calibration requirements and the time-division multiplexing approach. Once I grasped the concept, I added documentation comments to each block of code to clarify functionality. Then I implemented the missing features including frame rate control via knob when no buttons are pressed, Red/Green LED brightness control using A, B, and A+B button combinations, shared state management between UI and RGB structs, and dynamic tick time adjustment based on frame rate changes.

### How It Went

The biggest challenge was understanding the time-division multiplexing (TDM) concept initially. It took some time to grasp how rapidly switching between RGB channels creates the perception of mixed colors, and how the timing calculations work to maintain consistent brightness levels. Once I understood the underlying principles, implementing the button combinations and shared state management became more straightforward. The Embassy async framework made concurrent task management easier, though I find that debugging timing-sensitive embedded code requires patience.

### Observations

During calibration, I found that LED perception can be quite subjective and depends heavily on viewing angle and diffusion. The LED color changes slightly with different viewing angles, making it tricky to determine a "perfect" white balance. I experimented with different diffusion materials and settled on using a piece of Kleenex to get more consistent readings. I suspect our eye's sensitivity to different colors meant that relatively low green levels were needed to achieve perceived white light.

### Final Thought

This was a fun and engaging assignment that provided hands-on experience with embedded hardware including a breadboard and potentiometer control. It was interesting to see a simulated industry example. I think it demonstrates how real-world constraints often require creative software solutions when hardware designs aren’t optimal. I really appreciated the process of working through a realistic engineering problem from initial understanding through implementation to final results. The project combines embedded programming concepts with practical hardware interfacing, and I find it both educational and enjoyable.

----

*Original document below*

## Build and Run

Run with `cargo embed --release`. You'll need `cargo embed`, as
`cargo run` / `probe-rs run` does not reliably maintain a
connection for printing. See
https://github.com/probe-rs/probe-rs/issues/1235 for the
details.

## Wiring

Connect the RGB LED to the MB2 as follows:

* Red to P9 (GPIO1)
* Green to P8 (GPIO2)
* Blue to P16 (GPIO3)
* Gnd to Gnd

Connect the potentiometer (knob) to the MB2 as follows:

* Pin 1 to Gnd
* Pin 2 to P2
* Pin 3 to +3.3V

## UI

The knob controls the individual settings: frame rate and
color levels. Which parameter the knob controls should be
determined by which buttons are held. (Right now, the knob
jus always controls Blue. You should see the color change
from green to teal-blue as you turn the knob clockwise.)

* No buttons held: Change the frame rate in steps of 10
  frames per second from 10..160.
* A button held: Change the blue level from off to on over
  16 steps.
* B button held: Change the green level from off to on over
  16 steps.
* A+B buttons held: Change the red level from off to on over
  16 steps.

The "frame rate" (also known as the "refresh rate") is the
time to scan out all three colors. (See the scanout code.)
At 30 frames per second, every 1/30th of a second the LED
should scan out all three colors. If the frame rate is too
low, the LED will appear to "blink". If it is too high, it
will eat CPU for no reason.

I think the frame rate is probably set higher than it needs
to be right now: it can be tuned lower.

**LED Specifications**

[LED Wiring Diagram](https://docs.sunfounder.com/projects/sf-components/en/latest/component_rgb_led.html#:~:text=We%20use%20the%20common%20cathode%20one.&text=An%20RGB%20LED%20has%204,%2C%20GND%2C%20Green%20and%20Blue)
