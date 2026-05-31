#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::rcc::{AHBPrescaler, Hse, HseMode, Pll, PllMul, PllPDiv, PllPreDiv, PllSource, Sysclk};
use embassy_stm32::time::mhz;
use embassy_time::Timer;
use panic_halt as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Инициализируем периферию
    let p = embassy_stm32::init(init_config());

    // Инициализируем пин светодиода
    let mut led = Output::new(p.PA5, Level::Low, Speed::Low);

    loop {
        // "Мигаем" светодиодом
        led.toggle();

        // Теперь задержка таймера будет составлять ровно 1 секунду
        // в эмуляторе Renode, поскольку в .repl файле для платы Nucleo
        // статически указана частота для каждого таймера в 10 МГц.
        Timer::after_secs(1).await;
    }
}

fn init_config() -> Config {
    let mut config = Config::default();
    {
        // Используем внешний кварцевый резонатор на 8 МГц
        config.rcc.hse = Some(Hse {
            freq: mhz(8),
            mode: HseMode::Oscillator,
        });

        // ФАПЧ берет HSE в качестве опорной частоты
        config.rcc.pll_src = PllSource::HSE;

        // Фазовая автоподстройка частоты (ФАПЧ) - умножитель частоты: принимает
        // опорную частоту, например от внешнего генератора на 8 МГц и генерирует
        // все остальные тактовые частоты (даже в реалтайме), необходимые МК.
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4, // 8 MHz / 4 = 2 MHz
            mul: PllMul::MUL80, // 2 MHz * 80 = 160 MHz
            divp: Some(PllPDiv::DIV8), // 160 MHz / 8 = 20 MHz
            divq: None,
            divr: None,
        });

        // Задаем делитель для высокоскоростной системной шины AHB
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 20 MHz / 2 = 10 MHz

        // Выбираем системное тактирование, например от ФАПЧ,
        // теперь CPU микроконтроллера будет работать от PLL(ФАПЧ)
        config.rcc.sys = Sysclk::PLL1_P;
    }

    config
}