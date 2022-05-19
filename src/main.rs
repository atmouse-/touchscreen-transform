use std::fmt::{self, Formatter, Display};
use std::fs::File;
use std::io::{prelude::*, Bytes};
use std::fs::OpenOptions;
use evdev::{Device, Key, EventType, InputEvent};
use structopt::StructOpt;

#[macro_use]
extern crate log;
extern crate env_logger;

// trigger buttom
const SLOT_BUTTOM_CODE: u16 = 276;
//BTN_LEFT 272
//BTN_EXTRA 276

//#[derive(Debug)]
pub struct TransformInput {
    out_dev: File,
    max_x: u32,
    max_y: u32,
    abs_x: u32,
    abs_y: u32,
    abs_mt_tracking_id: i8,
}

impl TransformInput {
    pub fn add_x(&mut self, relx: i32) {
        let _x = self.abs_x as i32 + relx;
        if _x < 0 {
            self.abs_x = 0;
            return;
        }
        if _x > self.max_x as i32 {
            self.abs_x = self.max_x.to_owned();
            return;
        }
        self.abs_x = _x as u32
    }

    pub fn add_y(&mut self, rely: i32) {
        let _y = self.abs_y as i32 + rely;
        if _y < 0 {
            self.abs_y = 0;
            return;
        }
        if _y > self.max_y as i32 {
            self.abs_y = self.max_y.to_owned();
            return;
        }
        self.abs_y = _y as u32
    }

    pub fn handle(&mut self, event: InputEvent) -> () {
        match event.event_type() {
            EventType::RELATIVE => {
                match event.code() {
                    0 => {
                        // pos x
                        self.add_x(event.value())
                    },
                    1 => {
                        // pos y
                        self.add_y(event.value())
                    },
                    _ => {}
                }
                ;
            }
            EventType::KEY => {
                match event.code() {
                    SLOT_BUTTOM_CODE => {
                        match event.value() {
                            1 => {
                                // pressdown
                                self.new_tracking_id();
                                self.start_tracking();
                                self.send_abs_mt_position();
                                self.start_touch();
                                self.send_abs();
                                self.sync();
                            }
                            0 => {
                                // release
                                self.release_tracking_id();
                                self.stop_tracking();
                                self.stop_touch();
                                self.sync();
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                ;
            }
        }
    }

    pub fn write(&mut self, cmd: String) {
        // send to virtual_touchscreen https://github.com/vi/virtual_touchscreen/
        self.out_dev.write_all(cmd.as_bytes()).unwrap();
    }

    pub fn new_tracking_id(&mut self) {
        self.abs_mt_tracking_id = 64;
    }

    pub fn release_tracking_id(&mut self) {
        self.abs_mt_tracking_id = -1;
    }

    pub fn start_tracking(&mut self) {
        debug!("send ABS_MT_SLOT 0");
        debug!("send ABS_MT_TRACKING_ID {}", self.abs_mt_tracking_id);
        let cmd = format!("s 0\nT {}\n", self.abs_mt_tracking_id);
        self.write(cmd);
    }

    pub fn stop_tracking(&mut self) {
        debug!("send ABS_MT_TRACKING_ID -1");
        let cmd = format!("T -1\n");
        self.write(cmd);
    }

    pub fn start_touch(&mut self) {
        debug!("send BTN_TOUCH 1");
        debug!("send BTN_TOOL_FINGER 1");
        let cmd = format!("d 1\na 1\n");
        self.write(cmd);
    }

    pub fn stop_touch(&mut self) {
        debug!("send BTN_TOUCH 0");
        debug!("send BTN_TOOL_FINGER 0");
        let cmd = format!("d 0\na 0\n");
        self.write(cmd);
    }

    pub fn send_abs_mt_position(&mut self) {
        let x = self.abs_x;
        let y = self.abs_y;
        debug!("send ABS_MT_POSITION_X {}", x);
        debug!("send ABS_MT_POSITION_Y {}", y);
        let cmd = format!("X {}\nY {}\n", x, y);
        self.write(cmd);
    }
    pub fn send_abs(&mut self) {
        let x = self.abs_x;
        let y = self.abs_y;
        debug!("send ABS_X {}", x);
        debug!("send ABS_Y {}", y);
        let cmd = format!("x {}\ny {}\n", x, y);
        self.write(cmd);
    }

    pub fn sync(&mut self) {
        debug!("send SYN_REPORT 0");
        let cmd = format!("S 0\n");
        self.write(cmd);
        //self.out_dev.sync_all().unwrap();
    }
}


impl Display for TransformInput {
    // `f` is a buffer, and this method must write the formatted string into it
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let x = self.abs_x;
        let y = self.abs_y;

        // `write!` is like `format!`, but it will write the formatted string
        // into a buffer (the first argument)
        write!(f, "x:{} y:{}",
               x, y)
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "touchscreen-transform", about = "touchscreen-transform usage.")]
struct Opt {
    /// Activate debug mode
    // short and long flags (-d, --debug) will be deduced from the field's name
    #[structopt(short, long)]
    debug: bool,

    /// Max value for uinput ABS_X event
    #[structopt(short = "x", long = "abs-x-max")]
    abs_x_max: u32,

    /// Max value for uinput ABS_Y event
    #[structopt(short = "y", long = "abs-y-max")]
    abs_y_max: u32,

    /// select pointer device
    #[structopt(short = "i", long = "input-device")]
    input_device: String,

    /// output to virtual_touchscreen device
    #[structopt(short = "o", long = "virtual-touchscreen")]
    virtual_touchscreen: Option<String>,
}


fn main() {
    env_logger::init();

    let opt = Opt::from_args();
    let abs_x_max = opt.abs_x_max;
    let abs_y_max = opt.abs_y_max;
    let input_device = opt.input_device;
    let virtual_touchscreen = opt.virtual_touchscreen.unwrap_or(
        "/dev/virtual_touchscreen".to_string()
    );

    let mut in_device = Device::open(&input_device).unwrap();
    let mut out_device = OpenOptions::new()
            .write(true)
            .open(&virtual_touchscreen).unwrap();
    // disable grab
    //in_device.grab().unwrap();

    let mut vinput = TransformInput {
        out_dev: out_device,
        abs_mt_tracking_id: -1,
        max_x: abs_x_max,
        max_y: abs_y_max,
        abs_x: 0,
        abs_y: 0,
    };

    loop {
        in_device.fetch_events()
            .unwrap()
            .map(|ev| {
                //debug!("type={:?}, code={:?}, value={:?}", ev.event_type(), ev.code(), ev.value());
                //debug!("{:?}", ev);
                vinput.handle(ev);
                //debug!("{:?}", vinput);
            })
            .collect::<()>();

        // sync tracking event
        if vinput.abs_mt_tracking_id >= 0 {
            // in tracking_id

            /*
E: 0.506314 0003 0035 1756	# EV_ABS / ABS_MT_POSITION_X    1756
E: 0.506314 0003 0036 0886	# EV_ABS / ABS_MT_POSITION_Y    886
E: 0.506314 0003 0000 1756	# EV_ABS / ABS_X                1756
E: 0.506314 0003 0001 0886	# EV_ABS / ABS_Y                886
            */
            vinput.send_abs_mt_position();
            vinput.send_abs();
            vinput.sync();
        }
    }

}
