#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::hz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Channel;
use embassy_stm32::Config;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let ch3 = PwmPin::new_ch3(p.PB0, OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        p.TIM3,
        None,
        None,
        Some(ch3),
        None,
        hz(1),
        Default::default(),
    );
    let max = pwm.get_max_duty();
    pwm.enable(Channel::Ch3);

    info!("PWM initialized");
    info!("PWM max duty {}", max);

    pwm.set_duty(Channel::Ch3, max / 2);

    loop {
    }
}
