#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f1xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    // 拿到硬件外设抽象层句柄
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    // 拿到FLASH控制器句柄
    let mut flash = dp.FLASH.constrain();
    // 拿到RCC时钟句柄
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    // 拿到延时定时器句柄
    let mut delay = cp.SYST.delay(&clocks);

    // GPIOC 开启
    let mut gpioc = dp.GPIOC.split();
    // 将 PC13 配置为推挽输出
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    loop {
        // 翻转 LED 电平
        led.toggle();
        // 延时 500 毫秒
        delay.delay_ms(500_u32);
    }
}
