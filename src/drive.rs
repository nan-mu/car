use chrono::{DateTime, Local};
use log::info;
use log4rs::{append, config, encode, filter};
use rppal::gpio::{self, Gpio, Level, Result};
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

/// 包含电机的驱动和舵机的驱动。同时包含一个运动调度。
/// 这里所有的控制信号都是PWM，但由于需要使用耳机口放音乐，所以PWM信号只能找一个普通端口然后模拟输出PWM信号

/// 挡位
#[derive(Debug)]
pub enum Gear {
    Drift,      // 漂移，PWM=(0,0)
    Ahead(f64), // 前进，PWM=(1,0)，数值为油门，即使能通道的占空比，0.0~1.0
    Brake,      // 制动，PWM=(1,1)
}

#[derive(Debug)]
pub enum Diversion {
    Straight,
    Turn(u64),
}

#[derive(Debug)]
struct Times {
    reg: DateTime<Local>,
    start: DateTime<Local>,
    fine: DateTime<Local>,
}

#[derive(Debug)]
pub struct ControlMes {
    /// 控制模式
    mode: Gear,
    /// 转向角度
    diversion: Diversion,
    /// 控制信号持续时间
    duration: Duration,
    /// 任务的注册，运行，弹出（被打断和完整完成）的时间
    date: Times,
}

pub enum LaunchMode {
    Sleep,
    Debug,
    Brake,
    DeadWhell,
}

pub struct ControlManger {
    motor_pwm: (gpio::OutputPin, gpio::OutputPin), // 电机控制引脚，第一个为正向驱动引脚
    senvo_pwm: gpio::OutputPin,                    // 舵机控制引脚
    motor_tasks: VecDeque<ControlMes>,
    launch_mode: LaunchMode,
}

impl ControlManger {
    #[doc = "初始化电机舵机，主要是初始化引脚"]
    pub fn new(launch_mode: LaunchMode) -> Self {
        let stderr = append::console::ConsoleAppender::builder()
            .target(append::console::Target::Stderr)
            .build();
        let log_file = append::file::FileAppender::builder()
            .encoder(Box::new(encode::pattern::PatternEncoder::new(
                "[{l}][{d(%Y-%m-%d %H:%M:%S %Z)(local)}] - {m}\n",
            )))
            .build(format!(
                "log/CAR_{}.log",
                Local::now().format("%m-%d_%H-%M-%S")
            ))
            .unwrap();
        let log_config = config::runtime::Config::builder()
            .appender(config::Appender::builder().build("log_file", Box::new(log_file)))
            .appender(
                config::Appender::builder()
                    .filter(Box::new(filter::threshold::ThresholdFilter::new(
                        log::LevelFilter::Info,
                    )))
                    .build("stderr", Box::new(stderr)),
            )
            .build(
                config::Root::builder()
                    .appender("log_file")
                    .appender("stderr")
                    .build(log::LevelFilter::Trace),
            )
            .unwrap();
        let _handler = log4rs::init_config(log_config).unwrap();
        info!("初始化调度器");
        Self {
            motor_pwm: (
                Gpio::new().unwrap().get(20).unwrap().into_output_low(),
                Gpio::new().unwrap().get(16).unwrap().into_output_low(),
            ),
            senvo_pwm: Gpio::new().unwrap().get(23).unwrap().into_output(),
            motor_tasks: VecDeque::new(),
            launch_mode,
        }
    }
    #[doc = "启动调度器"]
    pub fn load_stats(&mut self) -> Result<()> {
        match self.launch_mode {
            LaunchMode::Debug => {
                info!("任务添加：调试模式");
                self.motor_tasks.push_back(ControlMes::new(
                    Gear::Ahead(0.4),
                    Diversion::Turn(1400),
                    Duration::from_secs(1),
                ));
            }
            LaunchMode::Brake => {
                info!("任务添加：制动模式");
                let killed_task = self.motor_tasks.pop_front().unwrap();
                let last_time = killed_task.duration
                    - killed_task
                        .date
                        .start
                        .signed_duration_since(Local::now())
                        .to_std()
                        .unwrap();
                self.motor_tasks.push_back(ControlMes::new(
                    //发出信号，没有添加花活
                    Gear::Brake,
                    Diversion::Straight,
                    Duration::from_secs(10),
                ));
                self.motor_tasks.push_back(ControlMes::new(
                    //发出信号
                    killed_task.mode,
                    Diversion::Straight,
                    last_time,
                ));
            }
            LaunchMode::DeadWhell => {
                //任务，160cm直线，80cm转弯180°，40cm转弯45°，40cm转弯180°，40cm转弯45°
                // self.motor_tasks.push_back(ControlMes::new(
                //     Gear::Ahead(0.3),
                //     Diversion::Straight,
                //     Duration::from_millis(3150),
                // ));
                // self.motor_tasks.push_back(ControlMes::new(
                //     Gear::Ahead(0.0),
                //     Diversion::Straight,
                //     Duration::from_millis(2000),
                // ));
                // self.motor_tasks.push_back(ControlMes::new(
                //     Gear::Ahead(0.0),
                //     Diversion::Turn(900),
                //     Duration::from_millis(2000),
                // ));
                // self.motor_tasks.push_back(ControlMes::new(
                //     Gear::Ahead(0.4),
                //     Diversion::Turn(1050),
                //     Duration::from_millis(2500),
                // ));
            }
            LaunchMode::Sleep => (),
        };
        Ok(())
    }
    pub fn launch(&mut self) {
        self.load_stats().unwrap();
        while !self.motor_tasks.is_empty() {
            let task = self
                .motor_tasks
                .front()
                .expect("运动任务列表已空但仍进入轮询");
            // let mental = thread::spawn(|| loop {
            //     let mental_pin = Gpio::new().unwrap().get(12).unwrap().into_input_pullup();
            //     match mental_pin.read() {
            //         Level::Low => {
            //             info!("找到金属！")
            //         }
            //         _ => (),
            //     }
            // });
            info!("发送电机执行任务：{:?}", task);
            run_motor(&mut self.motor_pwm, task).unwrap();
            run_senvo(&mut self.senvo_pwm, task).unwrap();
            thread::sleep(self.motor_tasks[0].duration);
            // mental.join().unwrap();
            self.motor_tasks.pop_front().expect("弹出任务失败");
        }
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
}

impl ControlMes {
    fn new(mode: Gear, diversion: Diversion, duration: Duration) -> Self {
        Self {
            mode,
            diversion,
            duration,
            date: Times {
                reg: Local::now(),
                start: Local::now(),
                fine: Local::now(),
            },
        }
    }
}

#[doc = "电机需要两路PWM，计划使用P20，P16，供电板上标记为P28,P2。然后控制使用百分比，因为这是油门的参数"]
fn run_motor(motor_pwm: &mut (gpio::OutputPin, gpio::OutputPin), mes: &ControlMes) -> Result<()> {
    match mes.mode {
        Gear::Drift => {
            motor_pwm.0.write(gpio::Level::Low);
            motor_pwm.1.write(gpio::Level::Low);
        }
        Gear::Ahead(accelerator) => {
            motor_pwm.0.set_pwm_frequency(50 as f64, accelerator)?;
            motor_pwm.1.write(gpio::Level::Low);
        }
        Gear::Brake => {
            motor_pwm.0.write(gpio::Level::High);
            motor_pwm.1.write(gpio::Level::High);
        }
    }
    Ok(())
}
#[doc = "控制电机只需要一路PWM，板载5V驱动，总共三个脚"]
fn run_senvo(senvo_pwm: &mut gpio::OutputPin, mes: &ControlMes) -> Result<()> {
    match mes.diversion {
        Diversion::Straight => {
            senvo_pwm.set_pwm(Duration::from_millis(20), Duration::from_micros(1250))
        }
        Diversion::Turn(direction) => senvo_pwm.set_pwm(
            Duration::from_millis(20),
            Duration::from_micros(direction), // 未完成
        ),
    }
}
