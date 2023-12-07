use num;
/// 包含电机的驱动和舵机的驱动。同时包含一个运动调度。
/// 这里所有的控制信号都是PWM，但由于需要使用耳机口放音乐，所以PWM信号只能找一个普通端口然后模拟输出PWM信号
use rppal::gpio::{self, Gpio, Result};
use std::time::Duration;

/// 挡位
enum Gear {
    Drift,        // 漂移，PWM=(0,0)
    Ahead(f64),   // 前进，PWM=(1,0)，数值为油门，即使能通道的占空比，0.0~1.0
    Reverse(f64), // 倒车，PWM=(0,1)，数值为油门，即使能通道的占空比，0.0~1.0
    Brake,        // 制动，PWM=(1,1)
}
enum Diversion {
    straight,
    turn(i8),
}

struct ControlMes {
    /// 控制模式
    mode: Gear,
    /// 转向角度
    diversion: Diversion,
    /// 控制信号持续时间
    duration: Duration,
}

struct ControlManger {
    motor_pwm: (gpio::OutputPin, gpio::OutputPin), // 电机控制引脚，第一个为正向驱动引脚
    senvo_pwm: gpio::OutputPin,                    // 舵机控制引脚
}

impl ControlManger {
    #[doc = "初始化电机舵机，主要是初始化引脚"]
    fn new() -> Self {
        Self {
            motor_pwm: (
                Gpio::new().unwrap().get(20).unwrap().into_output_low(),
                Gpio::new().unwrap().get(16).unwrap().into_output_low(),
            ),
            senvo_pwm: Gpio::new().unwrap().get(20).unwrap().into_output(),
        }
    }
    #[doc = "电机需要两路PWM，计划使用P20，P16，供电板上标记为P28,P2。然后控制使用百分比，因为这是油门的参数"]
    fn run_motor(&mut self, mes: ControlMes) -> Result<()> {
        match mes.mode {
            Gear::Drift => {
                self.motor_pwm.0.write(gpio::Level::Low);
                self.motor_pwm.1.write(gpio::Level::Low);
            }
            Gear::Ahead(accelerator) => {
                self.motor_pwm.0.set_pwm_frequency(50 as f64, accelerator)?;
                self.motor_pwm.1.write(gpio::Level::Low);
            }
            Gear::Reverse(accelerator) => {
                self.motor_pwm.1.set_pwm_frequency(50 as f64, accelerator)?;
                self.motor_pwm.0.write(gpio::Level::Low);
            }
            Gear::Brake => {
                self.motor_pwm.0.write(gpio::Level::High);
                self.motor_pwm.1.write(gpio::Level::High);
            }
        }
        Ok(())
    }
    #[doc = "控制电机只需要一路PWM，板载5V驱动，总共三个脚"]
    fn run_senvo(&mut self, mes: ControlMes) -> Result<()> {
        match mes.diversion {
            Diversion::straight => self
                .senvo_pwm
                .set_pwm(Duration::from_millis(20), Duration::from_micros(1250)),
            Diversion::turn(direction) => self.senvo_pwm.set_pwm(
                Duration::from_millis(20),
                Duration::from_micros(direction as u64), // 未完成
            ),
        }
    }
}
