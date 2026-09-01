#![no_std]

use bme280::i2c::BME280;
use rp235x_hal::{I2C, Timer, gpio, pac::I2C1, timer::CopyableTimer0};

struct BME280Data
{
    pub temperature: f32,
    pub pressure: f32,
    pub humidity: f32,
}

impl BME280Data
{
    pub fn new_null() -> BME280Data
    {
        return BME280Data
        {
            temperature: 0.0,
            pressure: 0.0,
            humidity: 0.0
        }
    }

    pub fn get_output_string(&self) -> String
    {
        return "\nTemperature: ".to_owned() + &self.temperature.to_string() + "\nPressure: " + &self.pressure.to_string()
        + "\nHumidity: " + &self.humidity.to_string();
    }
}

/*
The type parameters should be of the form rp235x_hal::gpio::bank0::GpioXX
*/
pub struct TRANTOR_BME280<I2CGpioPin1: rp235x_hal::gpio::PinId, I2CGpioPin2: rp235x_hal::gpio::PinId>
{
    bme: BME280<
            rp235x_hal::I2C<
                rp235x_hal::pac::I2C1,
                (
                    rp235x_hal::gpio::Pin<
                        I2CGpioPin1,
                        rp235x_hal::gpio::FunctionI2c,
                        rp235x_hal::gpio::PullUp,
                    >,
                    rp235x_hal::gpio::Pin<
                        I2CGpioPin2,
                        rp235x_hal::gpio::FunctionI2c,
                        rp235x_hal::gpio::PullUp,
                    >,
                ),
            >
        >,
    pub recent_data: BME280Data
}

impl<I2CGpioPin1: rp235x_hal::gpio::PinId, I2CGpioPin2: rp235x_hal::gpio::PinId> TRANTOR_BME280<I2CGpioPin1, I2CGpioPin2>
{
    pub fn new(i2c: I2C<I2C1, (
        gpio::Pin<I2CGpioPin1, gpio::FunctionI2c, gpio::PullUp>, 
        gpio::Pin<I2CGpioPin2, gpio::FunctionI2c, gpio::PullUp>
        )>) -> TRANTOR_BME280<I2CGpioPin1, I2CGpioPin2>
    {
        return TRANTOR_BME280
        {
            bme: BME280::new_primary(i2c),
            recent_data: BME280Data::new_null()
        }
    }

    pub fn init(&mut self, mut timer: Timer<CopyableTimer0>)
    {
        self.bme.init(&mut timer);
    }
}