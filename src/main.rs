#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::low_level::AFType;
use embassy_stm32::gpio::Speed;
use embassy_stm32::time::{hz, Hertz};
use embassy_stm32::timer::Channel;
use embassy_stm32::timer::*;
use embassy_stm32::{into_ref, PeripheralRef};
use embassy_stm32::{Config, Peripheral};
use {defmt_rtt as _, panic_probe as _};

mod util;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    PhaseShifted90Pwm::new(p.TIM8, p.PC6, p.PB14, hz(1));

    info!("PWM initialized");
}

pub struct PhaseShifted90Pwm<'d, T> {
    inner: PeripheralRef<'d, T>,
}

impl<'d, T: ComplementaryCaptureCompare16bitInstance> PhaseShifted90Pwm<'d, T> {
    /// Create a 90-degree phase shifted PWM driver.
    ///
    /// tim is configured in center aligned mode
    ///
    /// ch2n has 50% duty cycle
    ///
    /// ch1 is combined with ch5 to produce the phase shifted signal
    /// ch5 is set in PWM2 mode
    ///
    /// ```
    ///  CH2n OUTPUT :
    ///            ________          ________          ________          ____
    ///           |        |        |        |        |        |        |
    ///   ________|        |________|        |________|        |________|
    ///
    ///           | < TIMER RST     | < TIMER RST     | < TIMER RST
    ///
    ///  CH1 AND CH5 = CH1c OUTPUT (90-degree shift) :
    ///   ____          ________          ________          ________
    ///       |        |        |        |        |        |        |
    ///       |________|        |________|        |________|        |________
    ///
    ///  CH1 :
    ///   ____     _____________     _____________     _____________     ____
    ///       |___|             |___|             |___|             |___|
    ///
    ///  CH5 (PWM2 active low) :
    ///   ________      ____________      ____________      ____________
    ///           |____|            |____|            |____|            |____
    /// ```
    ///
    pub fn new(
        tim: impl Peripheral<P = T> + 'd,
        ch1: impl Peripheral<P = impl Channel1Pin<T>> + 'd,
        ch2n: impl Peripheral<P = impl Channel2ComplementaryPin<T>> + 'd,
        freq: Hertz,
    ) -> Self {
        into_ref!(tim, ch1, ch2n);

        T::enable_and_reset();

        ch1.set_speed(Speed::VeryHigh);
        ch1.set_as_af(ch1.af_num(), AFType::OutputPushPull);
        ch2n.set_speed(Speed::VeryHigh);
        ch2n.set_as_af(ch2n.af_num(), AFType::OutputPushPull);

        let mut this = Self { inner: tim };

        this.inner.set_counting_mode(CountingMode::EdgeAlignedUp); // Interrupts not used
        this.set_frequency(freq);
        this.inner.start();

        this.inner.enable_outputs();

        {
            // rust stm32 hal has no ch5 implementation, so do the setup manually
            let ptr = T::regs_gp16().as_ptr() as *mut u8;

            let ch1: usize = 0;
            let ch2: usize = 1;
            let ch5: usize = 4;

            util::ccmr_output(ptr, ch1 / 2)
                .modify(|w| w.set_ocm(ch1 % 2, OutputCompareMode::PwmMode1.into()));
            util::ccmr_output(ptr, ch2 / 2)
                .modify(|w| w.set_ocm(ch2 % 2, OutputCompareMode::PwmMode1.into()));
            util::ccmr_output(ptr, ch5 / 2)
                .modify(|w| w.set_ocm(ch5 % 2, OutputCompareMode::PwmMode2.into()));

            let max = this.get_max_duty() as f32;

            util::ccr(ptr, ch2).modify(|w| w.set_ccr((max * 0.50) as u16));
            util::ccr(ptr, ch1).modify(|w| w.set_ccr((max * 0.75) as u16));
            util::ccr(ptr, ch5).modify(|w| w.set_ccr_ch5((max * 0.25) as u16));

            util::ccr(ptr, ch5).modify(|w| w.set_ccr_group_ch5_ch1());

            this.enable(ch1);
            this.enable(ch2);
            this.enable(ch5);
        }

        this
    }

    pub fn set_frequency(&mut self, freq: Hertz) {
        self.inner.set_frequency(freq);
    }

    pub fn read_enable(&mut self, channel_index: usize) -> bool {
        let ptr = T::regs_advanced().as_ptr() as *mut u8;

        util::ccer(ptr).read().cce(channel_index)
    }

    pub fn enable(&mut self, channel_index: usize) {
        let ptr = T::regs_advanced().as_ptr() as *mut u8;

        util::ccer(ptr).modify(|w| w.set_cce(channel_index, true));
        util::ccer(ptr).modify(|w| w.set_ccne(channel_index, true));
    }

    pub fn get_max_duty(&self) -> u16 {
        self.inner.get_max_compare_value() + 1
    }

    pub fn set_duty(&mut self, channel: Channel, duty: u16) {
        assert!(duty <= self.get_max_duty());
        self.inner.set_compare_value(channel, duty)
    }
}
