#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::hz;
use embassy_stm32::timer::complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin};
use embassy_stm32::timer::simple_pwm::PwmPin;
use embassy_stm32::timer::*;
use embassy_stm32::Config;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let ch1 = PwmPin::new_ch1(p.PC6, OutputType::PushPull);
    let ch2 = ComplementaryPwmPin::new_ch2(p.PB14, OutputType::PushPull);

    let mut pwm = ComplementaryPwm::new(
        p.TIM8,
        Some(ch1),
        None,
        None,
        Some(ch2),
        None,
        None,
        None,
        None,
        hz(1),
        Default::default(),
    );
    let max = pwm.get_max_duty();

    pwm.enable(Channel::Ch1);
    pwm.enable(Channel::Ch2);

    info!("PWM initialized");
    info!("PWM max duty {}", max);

    pwm.set_duty(Channel::Ch1, max / 2);
    pwm.set_duty(Channel::Ch2, max / 2);
}
