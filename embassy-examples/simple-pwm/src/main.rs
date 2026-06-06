#![no_std]
#![no_main]

use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_stm32::{
    self as _, Config, gpio::OutputType, mode::Blocking, pac::{self}, time::{khz, mhz}, timer::{low_level::CountingMode, simple_pwm::{PwmPin, SimplePwm}}, usart::{Config as UartConfig, Uart}
};
use embassy_time::{Timer};
use heapless::String;
use panic_halt as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Инициализируем всю периферию
    let p = embassy_stm32::init(init_config());

    // Инициализируем UART
    let mut uart = Uart::new_blocking(
        p.USART2, 
        p.PA3, 
        p.PA2, 
        UartConfig::default()
    ).unwrap();

    // Инициализируем пин, который будет генерировать ШИМ сигнал.
    let ch1_pin = PwmPin::new(p.PB6, OutputType::PushPull);
    
    // Инициализируем простой драйвер генерации ШИМ-сигнала.
    let mut pwm = SimplePwm::new(
        p.TIM4, // таймер, от которого будет тактироваться ШИМ
        Some(ch1_pin), // пин первого канала таймера
        None, 
        None,
        None, 
        khz(1), // частота шим сигала
        CountingMode::EdgeAlignedUp
    );
    
    // Получаем канал таймера для управления ШИМ
    let mut ch1 = pwm.ch1();
    
    // Включаем канал
    ch1.enable();

    // Меняем регистры таймера, для предделителя 
    // и максимальной границы таймера
    pac::TIM4.psc().modify(|x| {*x = 0});
    pac::TIM4.arr().modify(|x| {x.set_arr(9999)});
    
    // Получаем максимальное значение скважности
    let max_duty = ch1.max_duty_cycle();
    
    let step = max_duty / 100;

    loop {
        // Плавное увеличение коэффициента заполнения с указанным шагом
        for i in 0..=100 {
            // Устанавливаем коэффициент заполнения 
            ch1.set_duty_cycle(i * step);
            // Выводим значения регистров таймера в UART для отладки
            write_registers_to_uart(&mut uart);
            Timer::after_millis(50).await;
        }

        // Плавное уменьшение коэффициента заполнения
        for i in (0..=100).rev() {
            ch1.set_duty_cycle(i * step);
            write_registers_to_uart(&mut uart);
            Timer::after_millis(50).await;
        }
    }
}

fn write_registers_to_uart(uart: &mut Uart<'_, Blocking>) {
    // Получаем регистры таймера для их вывода
    let psc = pac::TIM4.psc().read();
    let arr = pac::TIM4.arr().read().arr();
    let cnt = pac::TIM4.cnt().read().cnt();
    let ccr1 = pac::TIM4.ccr(0).read().ccr();
    
    let mut buffer: String<256> = String::new();

    writeln!(
        buffer, 
        "{{\"timer\":\"TIM4\",\"psc\":{},\"arr\":{},\"cnt\":{},\"channels\":[{},{},{},{}]}}",
        psc,
        arr,
        cnt,
        ccr1,
        0,
        0,
        0,
    ).unwrap();

    uart.blocking_write(buffer.as_bytes()).unwrap();
}

fn init_config() -> Config {
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
