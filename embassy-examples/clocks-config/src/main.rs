#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::{
    Config,
    rcc::{Hse, HseMode, Pll},
    time::mhz,
};
use embassy_stm32::rcc::{AHBPrescaler, APBPrescaler, PllMul, PllPDiv, PllPreDiv, PllQDiv, PllRDiv, PllSource, Sysclk};
use panic_halt as _;

// Если предделитель(prescaler) APB шины больше 1, то частоты таймеров,
// принадлежащих этой шине удваивается, например:
// APB1 = 45 MHz
// TIM2 = 90 MHz

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    // Использование внешнего кварцевого резонатора на 8 МГц
    config.rcc.hse = Some(Hse {
        freq: mhz(8),
        mode: HseMode::Oscillator,
    });

    // ФАПЧ берет HSE за опорную частоту
    config.rcc.pll_src = PllSource::HSE;

    // Фазовая автоподстройка частоты (ФАПЧ) - умножитель частоты: принимает
    // опорную частоту, например от внешнего генератора 8 МГц и генерирует
    // все остальные тактовые частоты (даже в реалтайме), необходимые МК.
    config.rcc.pll = Some(Pll {
        prediv: PllPreDiv::DIV4,
        mul: PllMul::MUL180,
        divp: Some(PllPDiv::DIV2),
        divq: Some(PllQDiv::DIV2),
        divr: Some(PllRDiv::DIV2),
    });

    // Выбираем системное тактирование, например от ФАПЧ,
    // теперь CPU будет работать от PLL(ФАПЧ)
    config.rcc.sys = Sysclk::PLL1_P;

    // Настройки предделителей(prescalers) частоты
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV4;
    config.rcc.apb2_pre = APBPrescaler::DIV2;

    let _ = embassy_stm32::init(config);
}