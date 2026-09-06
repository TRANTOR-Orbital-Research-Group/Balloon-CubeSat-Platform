use bme280::i2c::BME280;
use rp235x_hal::{I2C, Timer, gpio, pac::I2C1};
use core::fmt::Write;
use heapless::String;

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

    pub fn get_output_string(&self) -> heapless::String<128>
    {
        let mut output: String<128> = String::new();

        write!(output, "\nTemperature: {}\nPressure: {}\nHumidity: {}", &self.temperature, &self.pressure, &self.humidity)
            .expect("Create BME280 output message");

        return output;
    }
}

/*
The type parameters should be of the form rp235x_hal::gpio::bank0::GpioXX
*/
pub struct TRANTORBME280<I2CGpioPin1: rp235x_hal::gpio::PinId, I2CGpioPin2: rp235x_hal::gpio::PinId>
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

impl<I2CGpioPin1: rp235x_hal::gpio::PinId, I2CGpioPin2: rp235x_hal::gpio::PinId> TRANTORBME280<I2CGpioPin1, I2CGpioPin2>
{
    pub fn new(i2c: I2C<I2C1, (
        gpio::Pin<I2CGpioPin1, gpio::FunctionI2c, gpio::PullUp>, 
        gpio::Pin<I2CGpioPin2, gpio::FunctionI2c, gpio::PullUp>
        )>) -> TRANTORBME280<I2CGpioPin1, I2CGpioPin2>
    {
        return TRANTORBME280
        {
            bme: BME280::new_primary(i2c),
            recent_data: BME280Data::new_null()
        }
    }

    pub fn new_with_custom_address(i2c: I2C<I2C1, (
        gpio::Pin<I2CGpioPin1, gpio::FunctionI2c, gpio::PullUp>, 
        gpio::Pin<I2CGpioPin2, gpio::FunctionI2c, gpio::PullUp>
        )>, i2c_address: u8) -> TRANTORBME280<I2CGpioPin1, I2CGpioPin2>
    {
        return TRANTORBME280
        {
            bme: BME280::new(i2c, i2c_address),
            recent_data: BME280Data::new_null()
        }
    }

    pub fn init<CopyableTimer: rp235x_hal::timer::TimerDevice>(&mut self, mut timer: Timer<CopyableTimer>)
    {
        self.bme.init(&mut timer).expect("Initializing BME280");
    }

    pub fn record_data<CopyableTimer: rp235x_hal::timer::TimerDevice>(&mut self, mut timer: Timer<CopyableTimer>)
    {
        let measurements = self.bme.measure(&mut timer).expect("Getting the BME280 measurements.");

        self.recent_data.temperature = measurements.temperature;
        self.recent_data.pressure = measurements.pressure;
        self.recent_data.humidity = measurements.humidity;
    }
}