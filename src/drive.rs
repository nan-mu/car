#[warn(dead_code)]
/// 包含电机的驱动和舵机的驱动。同时包含一个运动调度。
/// 这里所有的控制信号都是PWM，但由于需要使用耳机口放音乐，所以PWM信号只能找一个普通端口然后模拟输出PWM信号
use rppal::gpio::{self, Gpio, Result};
use std::{thread, time::Duration};

/// 挡位
pub enum Gear {
    Drift,        // 漂移，PWM=(0,0)
    Ahead(f64),   // 前进，PWM=(1,0)，数值为油门，即使能通道的占空比，0.0~1.0
    Reverse(f64), // 倒车，PWM=(0,1)，数值为油门，即使能通道的占空比，0.0~1.0
    Brake,        // 制动，PWM=(1,1)
}
pub enum Diversion {
    Straight,
    Turn(u64),
}

pub struct ControlMes {
    /// 控制模式
    mode: Gear,
    /// 转向角度
    diversion: Diversion,
    /// 控制信号持续时间
    duration: Duration,
    /// 任务执行时间
}

pub enum LaunchMode {
    Sleep,
    Debug,
    DeadWhell,
}

pub struct ControlManger {
    motor_pwm: (gpio::OutputPin, gpio::OutputPin), // 电机控制引脚，第一个为正向驱动引脚
    senvo_pwm: gpio::OutputPin,                    // 舵机控制引脚
    motor_tasks: Vec<ControlMes>,
    launch_mode: LaunchMode,
}

impl ControlManger {
    #[doc = "初始化电机舵机，主要是初始化引脚"]
    pub fn new(launch_mode: LaunchMode) -> Self {
        Self {
            motor_pwm: (
                Gpio::new().unwrap().get(20).unwrap().into_output_low(),
                Gpio::new().unwrap().get(16).unwrap().into_output_low(),
            ),
            senvo_pwm: Gpio::new().unwrap().get(28).unwrap().into_output(),
            motor_tasks: vec![],
            launch_mode,
        }
    }
    #[doc = "启动调度器"]
    pub fn load_stats(&mut self) -> Result<()> {
        match self.launch_mode {
            LaunchMode::Debug => {
                self.motor_tasks.push(ControlMes::new(
                    Gear::Ahead(0.1),
                    Diversion::Turn(1250),
                    Duration::from_secs(10),
                ));
                self.motor_tasks.push(ControlMes::new(
                    Gear::Brake,
                    Diversion::Straight,
                    Duration::from_secs(10),
                ));
                self.motor_tasks.push(ControlMes::new(
                    Gear::Reverse(0.1),
                    Diversion::Turn(1450),
                    Duration::from_secs(10),
                ));
                self.motor_tasks.push(ControlMes::new(
                    Gear::Drift,
                    Diversion::Straight,
                    Duration::from_secs(10),
                ));
            }
            LaunchMode::DeadWhell => {}
            LaunchMode::Sleep => (),
        };
        Ok(())
    }
    pub fn launch(&mut self) {
        self.load_stats().unwrap();
        self.run_motor(&ControlMes::new(
            Gear::Ahead(1.0),
            Diversion::Turn(1250),
            Duration::from_secs(10),
        ))
        .unwrap();
        self.run_senvo(&ControlMes::new(
            Gear::Ahead(1.0),
            Diversion::Turn(1250),
            Duration::from_secs(10),
        ))
        .unwrap();
        thread::sleep(self.motor_tasks[0].duration);
    }
    pub fn reset(mut self) -> Self {
        self.motor_pwm = (
            Gpio::new().unwrap().get(20).unwrap().into_output_low(),
            Gpio::new().unwrap().get(16).unwrap().into_output_low(),
        );
        self.senvo_pwm = Gpio::new().unwrap().get(21).unwrap().into_output();
        self.launch_mode = LaunchMode::Sleep;
        self
    }
    #[doc = "电机需要两路PWM，计划使用P20，P16，供电板上标记为P28,P2。然后控制使用百分比，因为这是油门的参数"]
    fn run_motor(&mut self, mes: &ControlMes) -> Result<()> {
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
    fn run_senvo(&mut self, mes: &ControlMes) -> Result<()> {
        match mes.diversion {
            Diversion::Straight => self
                .senvo_pwm
                .set_pwm(Duration::from_millis(20), Duration::from_micros(1250)),
            Diversion::Turn(direction) => self.senvo_pwm.set_pwm(
                Duration::from_millis(20),
                Duration::from_micros(direction as u64), // 未完成
            ),
        }
    }
}

impl ControlMes {
    fn new(mode: Gear, diversion: Diversion, duration: Duration) -> Self {
        Self {
            mode,
            diversion,
            duration,
        }
    }
}

#[doc = "该函数计算路程所需的时间，具体内容有待实验数据"]
fn route2duration(lenght: u64) -> Duration {
    Duration::from_micros(lenght / 60)
}
