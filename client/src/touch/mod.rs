use std::{fs::File, str::FromStr, result};

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
    pub distance: Option<i32>,
    pub button: Option<i32>,
    pub stylus_back: Option<i32>,
    pub stylus_side: Option<i32>,
    pub stylus_tilt: Option<PixelSpaceCoord>,
}

/// Blocking event listener for touch events
pub struct TouchEventListener {
    device: Device,
}

fn open_device(path: String) -> std::io::Result<Device> {
    // Open the touch device
    let file = File::open(path)?;
    Device::new_from_file(file)
}

impl TouchEventListener {

    /// Construct a new `TouchEventListener` by opening the event stream
    pub fn open() -> std::io::Result<Self> {
        let touch_path: String = std::env::var("KOBO_TS_INPUT")
            .or(String::from_str("/dev/input/event1"))
            .unwrap();
        let device = open_device(touch_path)?;
        return Ok(Self { device })
    }

    pub fn open_input(touch_path: String) -> std::io::Result<Self> {
        let device = open_device(touch_path)?;
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
        wait_for_touch(timeout,  || self.next_raw_event())
    }

}


fn wait_for_touch(
    timeout: Option<Duration>,
    f: impl Fn() -> Result<(ReadStatus, InputEvent), std::io::Error>,
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
        let (status, event)= f().unwrap();
        println!("got event {}", event.event_code);

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
                position: PixelSpaceCoord::new(x.unwrap(), y.unwrap()),
                pressure: pressure.unwrap(),
                timestamp: Utc::now(),
                distance,
                button,
                stylus_back,
                stylus_side,
                stylus_tilt: if tilt_x.is_some() && tilt_y.is_some() { Some(PixelSpaceCoord::new(tilt_x.unwrap(), tilt_y.unwrap())) } else { None }
            });
        }
    }
}

mod tests {
    use std::time::Duration;

    use evdev_rs::enums::EventType;

    use crate::input::DeviceEvent;

    use super::*;

    #[test]
    fn test_touch() {

        use std::fs::File;
        use std::io::prelude::*;
        use evdev_rs::*;
        use evdev_rs::enums::*;

        //let listener = TouchEventListener::open().unwrap();
        use std::thread;

        // thread::spawn(move || {
        //     next_touch2(None, || {
        //         println!("simulate");
        //         if sent {
        //             return Err(std::io::Error::last_os_error());
        //         }
        //         sent = true;
        //         let time = TimeVal::new(1, 1);
        //         let code = EventCode::EV_SYN(EV_SYN::SYN_MT_REPORT);
        //         let evt = InputEvent::new(&time, &code, 0);
        //         return Ok((ReadStatus::Success, evt));
        //     });
        // });
        thread::sleep(Duration::from_secs(1));
        assert_eq!(true, true);
    }
}