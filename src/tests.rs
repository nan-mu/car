#[test]
fn test_drive() {
    let mut manger = crate::drive::ControlManger::new(crate::drive::LaunchMode::DeadWhell);
    manger.launch();
}

#[test]
fn test_servo() {
    use rppal::gpio::Gpio;
    use std::thread;
    // Retrieve the GPIO pin and configure it as an output.
    let mut pin = Gpio::new().unwrap().get(23).unwrap().into_output();

    //直线 1200
    pin.set_pwm(
        std::time::Duration::from_millis(20),
        std::time::Duration::from_micros(1200),
    )
    .unwrap();
    thread::sleep(std::time::Duration::from_secs(2));
}

#[test]
fn test_drv8701() {
    use rppal::gpio::Gpio;
    use std::{thread, time::Duration};
    let mut sd = (
        Gpio::new().unwrap().get(20).unwrap().into_output(),
        Gpio::new().unwrap().get(16).unwrap().into_output(),
    );
    sd.0.set_pwm_frequency(30 as f64, 0.5).unwrap(); // 20 接 PWM
    sd.1.set_low(); // 16 接 IO
    thread::sleep(Duration::from_secs(2));
}

#[test]
fn test_mental() {
    use rppal::gpio::Gpio;
    let mental = Gpio::new().unwrap().get(12).unwrap().into_input_pullup();
    loop {
        println!("{:?}", mental.read());
    }
}

// use rocket::{
//     http::{Request, Response},
//     routes,
// };

// use std::cmp::Ordering;
// use std::fs::read_to_string;
// use std::path::PathBuf;

// #[get("/")]
// fn index(req: Request) -> Response {
//     let mut log_files = vec![];
//     for entry in std::fs::read_dir("./log").unwrap() {
//         let path = entry.unwrap().path();
//         let filename = path.file_name().unwrap().to_str().unwrap();
//         let timestamp = filename[4..14].parse::<u64>().unwrap();
//         log_files.push((timestamp, path));
//     }

//     log_files.sort_by(|a, b| a.0.cmp(&b.0));

//     let (timestamp, log_file) = log_files.pop().unwrap();
//     let log_contents = read_to_string(log_file).unwrap();

//     Response::new(log_contents)
// }

// #[test]
// fn test_server() {
//     rocket::ignite().mount("/", routes![index]).launch();
// }
