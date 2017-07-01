//! Interrupt mapping and DMA channel setup.

use adc;
use aes;
use ast;
use cortexm4;
use crccu;
use dac;
use dma;
use flashcalw;
use gpio;
use i2c;
use kernel::Chip;
use kernel::common::{RingBuffer, Queue};
use nvic;
use pm;
use spi;
use trng;
use usart;

pub struct Sam4l {
    pub mpu: cortexm4::mpu::MPU,
    pub systick: &'static cortexm4::systick::SysTick,
}

const IQ_SIZE: usize = 100;
static mut IQ_BUF: [nvic::NvicIdx; IQ_SIZE] = [nvic::NvicIdx::HFLASHC; IQ_SIZE];
pub static mut INTERRUPT_QUEUE: Option<RingBuffer<'static, nvic::NvicIdx>> = None;


impl Sam4l {
    pub unsafe fn new() -> Sam4l {
        INTERRUPT_QUEUE = Some(RingBuffer::new(&mut IQ_BUF));

        usart::USART0.set_dma(&mut dma::DMA_CHANNELS[0], &mut dma::DMA_CHANNELS[1]);
        dma::DMA_CHANNELS[0].initialize(&mut usart::USART0, dma::DMAWidth::Width8Bit);
        dma::DMA_CHANNELS[1].initialize(&mut usart::USART0, dma::DMAWidth::Width8Bit);

        usart::USART1.set_dma(&mut dma::DMA_CHANNELS[2], &mut dma::DMA_CHANNELS[3]);
        dma::DMA_CHANNELS[2].initialize(&mut usart::USART1, dma::DMAWidth::Width8Bit);
        dma::DMA_CHANNELS[3].initialize(&mut usart::USART1, dma::DMAWidth::Width8Bit);

        usart::USART2.set_dma(&mut dma::DMA_CHANNELS[4], &mut dma::DMA_CHANNELS[5]);
        dma::DMA_CHANNELS[4].initialize(&mut usart::USART2, dma::DMAWidth::Width8Bit);
        dma::DMA_CHANNELS[5].initialize(&mut usart::USART2, dma::DMAWidth::Width8Bit);

        usart::USART3.set_dma(&mut dma::DMA_CHANNELS[6], &mut dma::DMA_CHANNELS[7]);
        dma::DMA_CHANNELS[6].initialize(&mut usart::USART3, dma::DMAWidth::Width8Bit);
        dma::DMA_CHANNELS[7].initialize(&mut usart::USART3, dma::DMAWidth::Width8Bit);

        spi::SPI.set_dma(&mut dma::DMA_CHANNELS[8], &mut dma::DMA_CHANNELS[9]);
        dma::DMA_CHANNELS[8].initialize(&mut spi::SPI, dma::DMAWidth::Width8Bit);
        dma::DMA_CHANNELS[9].initialize(&mut spi::SPI, dma::DMAWidth::Width8Bit);

        i2c::I2C0.set_dma(&dma::DMA_CHANNELS[10]);
        dma::DMA_CHANNELS[10].initialize(&mut i2c::I2C0, dma::DMAWidth::Width8Bit);

        i2c::I2C1.set_dma(&dma::DMA_CHANNELS[11]);
        dma::DMA_CHANNELS[11].initialize(&mut i2c::I2C1, dma::DMAWidth::Width8Bit);

        i2c::I2C2.set_dma(&dma::DMA_CHANNELS[12]);
        dma::DMA_CHANNELS[12].initialize(&mut i2c::I2C2, dma::DMAWidth::Width8Bit);

        adc::ADC0.set_dma(&dma::DMA_CHANNELS[13]);
        dma::DMA_CHANNELS[13].initialize(&mut adc::ADC0, dma::DMAWidth::Width16Bit);

        Sam4l {
            mpu: cortexm4::mpu::MPU::new(),
            systick: cortexm4::systick::SysTick::new(),
        }
    }
}

impl Chip for Sam4l {
    type MPU = cortexm4::mpu::MPU;
    type SysTick = cortexm4::systick::SysTick;

    fn service_pending_interrupts(&mut self) {
        use nvic::NvicIdx::*;

        unsafe {
            let iq = INTERRUPT_QUEUE.as_mut().unwrap();
            while let Some(interrupt) = iq.dequeue() {
                match interrupt {
                    ASTALARM => ast::AST.handle_interrupt(),

                    USART0 => usart::USART0.handle_interrupt(),
                    USART1 => usart::USART1.handle_interrupt(),
                    USART2 => usart::USART2.handle_interrupt(),
                    USART3 => usart::USART3.handle_interrupt(),

                    PDCA0 => dma::DMA_CHANNELS[0].handle_interrupt(),
                    PDCA1 => dma::DMA_CHANNELS[1].handle_interrupt(),
                    PDCA2 => dma::DMA_CHANNELS[2].handle_interrupt(),
                    PDCA3 => dma::DMA_CHANNELS[3].handle_interrupt(),
                    PDCA4 => dma::DMA_CHANNELS[4].handle_interrupt(),
                    PDCA5 => dma::DMA_CHANNELS[5].handle_interrupt(),
                    PDCA6 => dma::DMA_CHANNELS[6].handle_interrupt(),
                    PDCA7 => dma::DMA_CHANNELS[7].handle_interrupt(),
                    PDCA8 => dma::DMA_CHANNELS[8].handle_interrupt(),
                    PDCA9 => dma::DMA_CHANNELS[9].handle_interrupt(),
                    PDCA10 => dma::DMA_CHANNELS[10].handle_interrupt(),
                    PDCA11 => dma::DMA_CHANNELS[11].handle_interrupt(),
                    PDCA12 => dma::DMA_CHANNELS[12].handle_interrupt(),
                    PDCA13 => dma::DMA_CHANNELS[13].handle_interrupt(),
                    PDCA14 => dma::DMA_CHANNELS[14].handle_interrupt(),
                    PDCA15 => dma::DMA_CHANNELS[15].handle_interrupt(),

                    CRCCU => crccu::CRCCU.handle_interrupt(),

                    GPIO0 => gpio::PA.handle_interrupt(),
                    GPIO1 => gpio::PA.handle_interrupt(),
                    GPIO2 => gpio::PA.handle_interrupt(),
                    GPIO3 => gpio::PA.handle_interrupt(),
                    GPIO4 => gpio::PB.handle_interrupt(),
                    GPIO5 => gpio::PB.handle_interrupt(),
                    GPIO6 => gpio::PB.handle_interrupt(),
                    GPIO7 => gpio::PB.handle_interrupt(),
                    GPIO8 => gpio::PC.handle_interrupt(),
                    GPIO9 => gpio::PC.handle_interrupt(),
                    GPIO10 => gpio::PC.handle_interrupt(),
                    GPIO11 => gpio::PC.handle_interrupt(),

                    SPI => spi::SPI.handle_interrupt(),

                    TWIM0 => i2c::I2C0.handle_interrupt(),
                    TWIM1 => i2c::I2C1.handle_interrupt(),
                    TWIM2 => i2c::I2C2.handle_interrupt(),
                    TWIM3 => i2c::I2C3.handle_interrupt(),

                    TWIS0 => i2c::I2C0.handle_slave_interrupt(),
                    TWIS1 => i2c::I2C1.handle_slave_interrupt(),

                    HFLASHC => flashcalw::FLASH_CONTROLLER.handle_interrupt(),
                    ADCIFE => adc::ADC0.handle_interrupt(),
                    DACC => dac::DAC.handle_interrupt(),

                    TRNG => trng::TRNG.handle_interrupt(),
                    AESA => aes::AES.handle_interrupt(),
                    _ => {}
                }
                nvic::enable(interrupt);
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { INTERRUPT_QUEUE.as_mut().unwrap().has_elements() }
    }

    fn mpu(&self) -> &cortexm4::mpu::MPU {
        &self.mpu
    }

    fn systick(&self) -> &cortexm4::systick::SysTick {
        self.systick
    }

    fn prepare_for_sleep(&self) {
        unsafe {
            gpio::PB[15].enable();
            gpio::PB[15].enable_output();
            gpio::PB[15].toggle();
            gpio::PB[15].toggle();
            gpio::PB[15].toggle();
        }
        if pm::deep_sleep_ready() {
            unsafe {
                gpio::PB[15].set();
                cortexm4::scb::set_sleepdeep();
            }
        } else {
            unsafe {
                gpio::PB[15].clear();
                cortexm4::scb::unset_sleepdeep();
            }
        }
    }

    fn wake_up(&self) {
        unsafe{
            gpio::PB[15].enable();
            gpio::PB[15].enable_output();
            gpio::PB[15].toggle();
            gpio::PB[15].toggle();
            gpio::PB[15].toggle();
            gpio::PB[15].toggle();
            gpio::PB[15].toggle();
            gpio::PB[15].toggle();
            gpio::PB[15].toggle();
            gpio::PB[15].toggle();
        }
    }
}
