#![no_std]
#![no_main]

use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_stm32::{
    self as _, Config,
    adc::{Adc, AdcConfig, Resolution, SampleTime},
    time::mhz,
    usart::{Config as UartConfig, Uart},
};
use embassy_time::Timer;
use heapless::String;
use panic_halt as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(init_peripheral_config());

    let mut uart = Uart::new_blocking(p.USART2, p.PA3, p.PA2, UartConfig::default()).unwrap();

    let mut adc = Adc::new_with_config(
        p.ADC1,
        AdcConfig {
            resolution: Some(Resolution::BITS12),
        },
    );

    let mut pin = p.PA0;

    loop {
        let value = adc.blocking_read(&mut pin, SampleTime::CYCLES112);

        let mut msg: String<64> = String::new();

        writeln!(msg, "ADC: {}", value).unwrap();
        uart.blocking_write(msg.as_bytes()).unwrap();

        Timer::after_millis(300).await;
    }
}

fn init_peripheral_config() -> Config {
    let mut config = Config::default();

    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: mhz(8),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL80,
            divp: Some(PllPDiv::DIV8),
            divq: None,
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
    }

    config
}
