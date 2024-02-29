#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Ipv4Address, Stack, StackResources};
use embassy_stm32::eth::generic_smi::GenericSMI;
use embassy_stm32::eth::{Ethernet, PacketQueue};
use embassy_stm32::peripherals::ETH;
use embassy_stm32::rng::Rng;
use embassy_stm32::{bind_interrupts, eth, peripherals, rng, Config};
use embassy_time::{Duration, Timer, TICK_HZ};
use embedded_io_async::Write;
use rand_core::RngCore;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

mod util;

bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

type Device = Ethernet<'static, ETH, GenericSMI>;

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<Device>) -> ! {
    stack.run().await
}

#[embassy_executor::task]
async fn log_idle_time() -> ! {
    /// 1000 is a scale factor, defmt macro should be adjusted according to it's length
    /// rounding + avoiding floating point operations
    ///
    /// XXX: as it's not an interrupt executor, tasks that take more than one second won't
    /// display load correctly
    const SCALE: i32 = 1000;
    loop {
        unsafe {
            let idle_ticks = util::EXECUTOR.as_mut().unwrap().idle_duration.as_ticks();
            let cpu_load = (SCALE * (TICK_HZ - idle_ticks) as i32) / TICK_HZ as i32;
            info!("CPU load: 0.{:03}", cpu_load);
            util::EXECUTOR.as_mut().unwrap().idle_duration = Duration::from_ticks(0);
            Timer::after_secs(1).await;
        }
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let e = util::Executor::take();
    e.run(|spawner| {
        unwrap!(spawner.spawn(async_main(spawner)));
        unwrap!(spawner.spawn(log_idle_time()));
    })
}

#[embassy_executor::task]
async fn async_main(spawner: Spawner) -> ! {
    info!("Hello world");

    #[cfg(not(debug_assertions))]
    fn example() {
        info!("Enabling caches for release build");
        let mut cp = cortex_m::Peripherals::take().unwrap();
        cp.SCB.enable_icache();
        cp.SCB.enable_dcache(&mut cp.CPUID);
    }

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.hsi48 = Some(Default::default()); // needed for RNG
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
    }
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    // Generate random seed.
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut seed = [0; 8];
    rng.fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];

    static PACKETS: StaticCell<PacketQueue<4, 4>> = StaticCell::new();
    let device = Ethernet::new(
        PACKETS.init(PacketQueue::<4, 4>::new()),
        p.ETH,
        Irqs,
        p.PA1,
        p.PA2,
        p.PC1,
        p.PA7,
        p.PC4,
        p.PC5,
        p.PG13,
        p.PB13,
        p.PG11,
        GenericSMI::new(0),
        mac_addr,
    );

    let config = embassy_net::Config::dhcpv4(Default::default());

    // Init network stack
    static STACK: StaticCell<Stack<Device>> = StaticCell::new();
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let stack = &*STACK.init(Stack::new(
        device,
        config,
        RESOURCES.init(StackResources::<3>::new()),
        seed,
    ));

    // Launch network task
    unwrap!(spawner.spawn(net_task(&stack)));

    // Ensure DHCP configuration is up before trying connect
    stack.wait_config_up().await;

    info!("Network task initialized");

    const BUFSIZE: usize = 8760;

    let mut tx_buffer = [0u8; BUFSIZE];
    let mut deadbeef_buffer = [0u8; BUFSIZE];
    let mut rx_buffer = [0u8; 1460];

    // Fill the tx_buffer with the pattern 0xDEADBEEF
    for chunk in deadbeef_buffer.chunks_mut(4) {
        chunk.copy_from_slice(&0xDEADBEEFu32.to_ne_bytes());
    }

    let total_size_to_send = 10_000_000; // 10 MB in bytes
    let mut total_sent = 0usize;

    loop {
        let mut socket = TcpSocket::new(&stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        let remote_endpoint = (Ipv4Address::new(192, 168, 88, 69), 8000);
        info!("connecting...");
        let r = socket.connect(remote_endpoint).await;
        if let Err(e) = r {
            info!("connect error: {:?}", e);
            Timer::after_secs(1).await;
            continue;
        }
        info!("connected!");

        while total_sent < total_size_to_send {
            let r = socket.write_all(&deadbeef_buffer).await;
            if let Err(e) = r {
                info!("write error: {:?}", e);
                break;
            }
            total_sent += BUFSIZE;
        }

        if total_sent >= total_size_to_send {
            info!("Successfully sent 10 MB of data");
            socket.close();
        }

        total_sent = 0;
        Timer::after_millis(1500).await;
    }
}
