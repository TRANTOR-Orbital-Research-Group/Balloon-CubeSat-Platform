use usbd_serial::SerialPort;
use rp235x_hal::usb::UsbBus;

fn handle_result<Ok_T, Error_T>(res: Result<Ok_T, Error_T>, error_message: &str, mut serial: SerialPort<'static, rp235x_hal::usb::UsbBus>)
{
    match res
    {
        Ok(o) => (),
        Err(e) =>
        {
            serial.write(error_message.as_bytes()).expect("Display error message over serial");
        }
    }
}