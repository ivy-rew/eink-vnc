use std::{fs::File, str::FromStr, result};

use chrono::{DateTime, Duration, Utc};
use evdev_rs::{Device, InputEvent, ReadFlag, ReadStatus};

/// Describes a touch event.
#[derive(Debug, Clone)]
pub struct Touch {
    /// The touch position
    pub position: Coord,
    /// The touch pressure
    pub pressure: i32,
    /// The timestamp of the touch event
    pub timestamp: DateTime<Utc>,
    pub distance: Option<i32>,
    pub button: Option<i32>,
    pub stylus_back: Option<i32>,
    pub stylus_side: Option<i32>,
    pub stylus_tilt: Option<Coord>,
}

#[derive(Debug, Clone)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

/// Blocking event listener for touch events
pub struct TouchEventListener {
    device: Device,
}

impl TouchEventListener {

    /// Construct a new `TouchEventListener` by opening the event stream
    pub fn open() -> std::io::Result<Self> {
        let touch_path: String = std::env::var("KOBO_TS_INPUT")
            .or(String::from_str("/dev/input/event1"))
            .unwrap();
        let device = Self::open_device(touch_path)?;
        return Ok(Self { device })
    }

    pub fn open_input(touch_path: String) -> std::io::Result<Self> {
        let device = Self::open_device(touch_path)?;
        Ok(Self { device })
    }

    fn open_device(path: String) -> std::io::Result<Device> {
        // Open the touch device
        let file = File::open(path)?;
        Device::new_from_file(file)
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
        let mut stylus_back: Option<i32> = None;
        let mut stylus_side: Option<i32> = None;
        let mut syn: Option<i32> = None;
        let mut distance: Option<i32> = None;
        let mut tilt_x = None;
        let mut tilt_y = None;

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
                        evdev_rs::enums::EV_ABS::ABS_TILT_X => {
                            tilt_x = Some(event.value);
                        }
                        evdev_rs::enums::EV_ABS::ABS_TILT_Y => {
                            tilt_y = Some(event.value);
                        }
                        _ => {}
                    },
                    evdev_rs::enums::EventCode::EV_KEY(kind) => match kind {
                        evdev_rs::enums::EV_KEY::BTN_TOUCH => {
                            button = Some(event.value);
                        }
                        evdev_rs::enums::EV_KEY::BTN_STYLUS => {
                            stylus_back = Some(event.value);
                        }
                        evdev_rs::enums::EV_KEY::BTN_STYLUS2 => {
                            stylus_side = Some(event.value);
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
                    position: Coord{x: x.unwrap(), y: y.unwrap()},
                    pressure: pressure.unwrap(),
                    timestamp: Utc::now(),
                    distance,
                    button,
                    stylus_back,
                    stylus_side,
                    stylus_tilt: if tilt_x.is_some() && tilt_y.is_some() { Some(Coord{x: tilt_x.unwrap(), y: tilt_y.unwrap()}) } else { None }
                });
            }
        }
    }
}
