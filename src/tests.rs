// use rppal::gpio::Gpio;

#[test]
fn test_drive() {
    let mut manger = crate::drive::ControlManger::new(crate::drive::LaunchMode::Debug);
    manger.launch();
}

#[test]
fn test_pwm() {
    //需要测试多组频率下电机控制器的表现
    use rppal::gpio::Gpio;
    let mut sd = (
        Gpio::new().unwrap().get(20).unwrap().into_output(),
        Gpio::new().unwrap().get(16).unwrap().into_output(),
    );
    sd.0.set_pwm_frequency(30 as f64, 0.5).unwrap();
    sd.1.set_pwm_frequency(30 as f64, 0.0).unwrap();
}

#[test]
fn test_drv8701() {
    use rppal::gpio::Gpio;
    let mut sd = (
        Gpio::new().unwrap().get(20).unwrap().into_output(),
        Gpio::new().unwrap().get(16).unwrap().into_output(),
    );
    sd.0.set_pwm_frequency(30 as f64, 0.5).unwrap(); // 20 接 PWM
    sd.1.set_low(); // 16 接 IO
}

// #[test]
// fn test_i2c() {
//     use rppal::{gpio::Gpio, i2c::I2c};
//     let mut sd = Gpio::new().unwrap().get(4).unwrap();
//     sd.into_output_low();
//     let mut iic = I2c::new().unwrap();
//     iic.set_slave_address(0x2b).unwrap();
//     let u16_to_u8s = |x: u16| [x as u8, (x >> 8) as u8];
//     let load = |add: u8, val: u16| [add, val as u8, (val >> 8) as u8];
//     iic.write(&load(0x08, 0x04d6)).unwrap();
//     iic.write(&load(0x10, 0x000a)).unwrap();
//     iic.write(&load(0x14, 0x1002)).unwrap();
//     iic.write(&load(0x19, 0x0000)).unwrap();
//     iic.write(&load(0x1b, 0x020c)).unwrap();
//     iic.write(&load(0x1e, 0x9000)).unwrap();
//     iic.write(&load(0x1a, 0x1601)).unwrap();
//     let mut DATAx_MSB = [0u8; 2];
//     let mut DATAx_LSB = [0u8; 2];
//     iic.block_read(0x00, &mut DATAx_MSB).unwrap();
//     iic.block_read(0x01, &mut DATAx_LSB).unwrap();
//     println!("{:?}, {:?}", DATAx_MSB, DATAx_LSB);
// }

// #[test]
// fn test_mental() {
//     // use crate::metal_sensor as ldc;
//     use rppal::i2c::I2c;
//     // use std::str::FromStr;
//     let mut iic = I2c::new().unwrap();
//     iic.set_slave_address(0x2B).unwrap();
//     let mut ldc = ldc::Ldc::new(iic);
//     // ldc.reset().unwrap();

//     let div = ldc::Fsensor::from_inductance_capacitance(12.583, 100.0).to_clock_dividers(None);

//     //set clock dividers
//     ldc.set_clock_dividers(ldc::Channel::Zero, div).unwrap();
//     ldc.set_conv_settling_time(ldc::Channel::Zero, 40).unwrap();
//     ldc.set_ref_count_conv_interval(ldc::Channel::Zero, 0x0546)
//         .unwrap();
//     ldc.set_sensor_drive_current(ldc::Channel::Zero, 0b01110)
//         .unwrap();

//     ldc.set_mux_config(
//         ldc::MuxConfig::default()
//             .with_auto_scan(true)
//             .with_deglitch_filter_bandwidth(ldc::Deglitch::ThreePointThreeMHz),
//     )
//     .unwrap();
//     ldc.set_config(ldc::Config::default()).unwrap();
//     ldc.set_error_config(
//         ldc::ErrorConfig::default().with_amplitude_high_error_to_data_register(true),
//     )
//     .unwrap();

//     // timing ignored because polling with a cp2112 with no delays is slow enough already
//     // outputting just newline separated numbers so you can feed it into https://github.com/mogenson/ploot
//     loop {
//         println!(
//             "{} {}",
//             ldc.read_data_24bit(ldc::Channel::Zero).unwrap(),
//             ldc.read_data_24bit(ldc::Channel::One).unwrap(),
//         );
//     }
// }
