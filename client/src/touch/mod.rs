use std::{fs::File, str::FromStr};

use chrono::{DateTime, Duration, Utc};
use evdev_rs::{Device, InputEvent, ReadFlag, ReadStatus};
use nalgebra::distance;

mod coords;
mod mouse;
pub use self::coords::PixelSpaceCoord;
pub use self::mouse::{MOUSE_LEFT, MOUSE_UNKNOWN, mouse_btn_to_vnc};

/// Describes a touch event.
#[derive(Debug, Clone)]
pub struct Touch {
    /// The touch position
    pub position: PixelSpaceCoord,
    /// The touch pressure
    pub pressure: i32,
    /// The timestamp of the touch event
    pub timestamp: DateTime<Utc>,

    pub button: Option<i32>,
    pub distance: Option<i32>,
}

/// Blocking event listener for touch events
pub struct TouchEventListener {
    device: Device,
}

impl TouchEventListener {

    /// Construct a new `TouchEventListener` by opening the event stream
    pub fn open() -> std::io::Result<Self> {
        // Open the touch device
        let touch_path: String = std::env::var("KOBO_TS_INPUT")
            .or(String::from_str("/dev/input/event1"))
            .unwrap();
        let file = File::open(touch_path)?;
        let device = Device::new_from_file(file)?;

        Ok(Self { device })
    }

    /// Read the next event from the stream
    pub fn next_raw_event(&self) -> std::io::Result<(ReadStatus, InputEvent)> {
        self.device
            .next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING)
    }

    /// Read the next touch event
    ///
    /// ## Notes
    ///
    /// While this will attempt to stop at a timeout, there is a chance the event stream will just block us past it anyways.
    ///
    /// Pressure is currently unreliable, so we'll just assume it's always down.
    pub fn next_touch(
        &self,
        timeout: Option<Duration>,
    ) -> Option<Touch> {
        // Keep track of the start time
        let start = Utc::now();

        // Holder for out data
        let mut x = None;
        let mut y = None;
        let mut pressure = None;
        let mut button: Option<i32> = None;
        let mut syn: Option<i32> = None;
        let mut distance: Option<i32> = None;

        // Loop through the incoming event stream
        loop {
            // Check the timeout
            if let Some(timeout) = timeout {
                let elapsed = Utc::now().signed_duration_since(start);
                if elapsed > timeout {
                    return None;
                }
            }

            // Read the next event
            let (status, event) = self.next_raw_event().unwrap();

            // Check the status
            if status == ReadStatus::Success {
                // We are looking for ABS touch events
                match event.event_code {
                    evdev_rs::enums::EventCode::EV_ABS(kind) => match kind {
                        evdev_rs::enums::EV_ABS::ABS_X => {
                            x = Some(event.value);
                        }
                        evdev_rs::enums::EV_ABS::ABS_Y => {
                            y = Some(event.value);
                        }
                        evdev_rs::enums::EV_ABS::ABS_MT_POSITION_X => {
                            x = Some(event.value);
                        }
                        evdev_rs::enums::EV_ABS::ABS_MT_POSITION_Y => {
                            y = Some(event.value);
                        }
                        evdev_rs::enums::EV_ABS::ABS_PRESSURE => {
                            pressure = Some(event.value);
                        }
                        evdev_rs::enums::EV_ABS::ABS_MT_PRESSURE => {
                            pressure = Some(event.value);
                        }
                        evdev_rs::enums::EV_ABS::ABS_MT_DISTANCE => {
                            distance = Some(event.value);
                        }
                        _ => {}
                    },
                    evdev_rs::enums::EventCode::EV_KEY(kind) => match kind {
                        evdev_rs::enums::EV_KEY::BTN_TOUCH => {
                            button = Some(event.value);
                        }
                        _ => {}
                    },
                    evdev_rs::enums::EventCode::EV_SYN(kind) => match kind {
                        evdev_rs::enums::EV_SYN::SYN_REPORT => {
                            syn = Some(event.value); // mouse state complete 
                        }
                        _ => {}
                    },
                    _ => { /* Unused */ }
                }
            }

            // Check if we have all the data
            if syn.is_some() {
                // Return the touch event
                return Some(Touch {
                    position: PixelSpaceCoord::new(x.unwrap(), y.unwrap()),
                    pressure: pressure.unwrap(),
                    timestamp: Utc::now(),
                    button,
                    distance
                });
            }
        }
    }
}
