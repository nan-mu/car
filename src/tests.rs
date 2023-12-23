// use rppal::gpio::Gpio;

use chrono::Duration;

#[test]
fn test_drive() {
    let mut manger = crate::drive::ControlManger::new(crate::drive::LaunchMode::DeadWhell);
    manger.launch();
}

#[test]
fn test_mental() {
    use rppal::gpio::Gpio;
    let mental = Gpio::new().unwrap().get(12).unwrap().into_input_pullup();
    loop {
        println!("{:?}", mental.read());
    }
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
    thread::sleep(std::time::Duration::from_secs(5));
    //右转 1400
    pin.set_pwm(
        std::time::Duration::from_millis(20),
        std::time::Duration::from_micros(1400),
    )
    .unwrap();
    thread::sleep(std::time::Duration::from_secs(5));
    //左转 1000
    pin.set_pwm(
        std::time::Duration::from_millis(20),
        std::time::Duration::from_micros(1000),
    )
    .unwrap();
    thread::sleep(std::time::Duration::from_secs(5));
    pin.set_pwm(
        std::time::Duration::from_millis(20),
        std::time::Duration::from_micros(1200),
    )
    .unwrap();
    thread::sleep(std::time::Duration::from_secs(1));
}
