use embedded_hal::i2c::{Error, SevenBitAddress, I2c};
use embedded_hal_bus::i2c::RefCellDevice;
use ublox::{UbxParserIter, nav_pvt};
use ublox::{Parser, UbxPacketMeta, FixedLinearBuffer, UbxPacket, proto23::PacketRef, GnssFixType, nav_status::NavStatusFlags}; 
use heapless::String;
use core::convert::TryFrom;
use core::fmt::Write;
use ublox::nav_pvt::proto23::NavPvtRef;

struct UbloxGPSData
{
    fix_type: GnssFixType,
    num_satellites: u8,
    lattitude: f64,
    longitude: f64,
    horizontal_accuracy: f64,
    millis_since_gps_week: u32,
    status_flags: NavStatusFlags,
    unknown_packet: bool,
    noise_per_ms: u16,
}

pub struct UbloxGPS<'a, I2C>
{
    i2c: &'a mut I2C,
    address: u8,
    other_msg: String<64>,
    pub data: UbloxGPSData
}

impl<I2C: I2c<SevenBitAddress>> UbloxGPS<'static, I2C>
{
    pub fn new(i2c_address: u8, i2c: &'static mut I2C) -> UbloxGPS<'static, I2C>
    {
        // Enable navigation packets
        i2c.write(i2c_address, &[0xB5, 0x62, 0x06, 0x01, 0x03, 0x00, 0x01, 0x07, 0x01,
            0x13, 0x51])
            .expect("Enable navigation packets");

        // Start antenna
        i2c.write(i2c_address, &[0xB5, 0x62, 0x06, 0x13, 0x04, 0x00, 0x1F, 0x00, 0xF0, 0x7D, 0xAB, 0x1F])
            .expect("Start antenna");


        return UbloxGPS
        {
            i2c,
            address: i2c_address,
            //data_msg: String::new(),
            // info_msg: String::new(),
            other_msg: String::new(),
            data: UbloxGPSData
            {
                fix_type: GnssFixType::NoFix,
                num_satellites: 0u8,
                lattitude: 0.0f64,
                longitude: 0.0f64,
                horizontal_accuracy: 0.0f64,
                millis_since_gps_week: 0u32,
                status_flags: NavStatusFlags::empty(),
                unknown_packet: false,
                noise_per_ms: 0u16
            }
        };
    }

    pub fn update(&mut self, buffer: &mut [u8; 1024])
    {
        let mut parser: Parser<FixedLinearBuffer<'_>, _> = Parser::new(FixedLinearBuffer::new(buffer));

        // The following code is reused from Sam (IntentionalDisaster99) and modified for this abstraction

        // The two bytes that represent the length (we will send it as a buffer so that I2C can tell us how much data is being sent) 
        let mut length_bytes = [0u8; 2];

        // Telling the GPS that we want data and asking it how much it is ready to send
        if self.i2c.write_read(self.address, &[0xFD], &mut length_bytes).is_ok() {

            // How much data it is ready to send
            let bytes_available = u16::from_be_bytes(length_bytes) as usize;

            // A running total of how many bytes we have left to read so we can iterate across them
            let mut total_to_read = bytes_available;

            // Iterating over all of the bytes
            while total_to_read > 0 {
                
                // Making sure that the maximum amount of data we get will still fit in our buffer
                let mut data_chunk = [0u8; 64]; 
                let to_read = core::cmp::min(total_to_read, data_chunk.len());
                        
                // Asking for the information from the gps
                if self.i2c.write_read(self.address, &[0xFF], &mut data_chunk[..to_read]).is_ok() {

                    // Passing the stuff to the parser
                    let mut it: UbxParserIter<'_, FixedLinearBuffer> = parser.consume_ubx(&data_chunk[..to_read]);

                    // Iterating through the data that the parser has to read the packets
                    while let Some(packet_result) = it.next() {

                        // If we have a packet, we need to read it
                        if let Ok(packet) = packet_result {

                            // Matching the protocol of the packet (this is technically not needed, but I was testing with a GPS that
                            // uses a different protocol than the M9N, so this might help prevent headaches later
                            if let UbxPacket::Proto23(p) = packet {
                                
                                // Now that we know that we have the packet, we need to know which kind
                                match p {

                                        // The NavPvt packet is the navagation packet we want
                                        PacketRef::NavPvt(nav_pvt) => 
                                        {
                                            self.data.fix_type = nav_pvt.fix_type();
                                            self.data.horizontal_accuracy = nav_pvt.horizontal_accuracy();
                                            self.data.lattitude = nav_pvt.latitude();
                                            self.data.longitude = nav_pvt.longitude();
                                            self.data.num_satellites = nav_pvt.num_satellites();

                                            self.data.unknown_packet = false;
                                        },
                                        // This is a packet that says the status of the GPS, so like the status and any errors
                                        PacketRef::NavStatus(status) => 
                                        {
                                            self.data.millis_since_gps_week = status.itow();
                                            self.data.status_flags = status.flags();

                                            self.data.unknown_packet = false;
                                        },
                                        // If there are any unknown packets, this is where we can catch them
                                        // in my testing I didn't actually ever get one though
                                        PacketRef::Unknown(_) =>  
                                        {
                                            self.data.unknown_packet = true;
                                        },
                                        // This is the hardware status packet, you have to specifically request it 
                                        // but you can use the noise to tell you how close you are to getting a lock
                                        // Noise below 90 will quickly get a lock, slowly between 90 and 100, and rarely above 100
                                        PacketRef::MonHw(hw) => 
                                        {
                                            self.data.noise_per_ms = hw.noise_per_ms();
                                        },
                                        // The wildcard, so any other packet that we don't have accounted for 
                                        _ => {
                                            // Printing out what the ID and class of the packet is 
                                            let (class, id) = p.class_and_msg_id();
                                            let mut debug_msg = String::<64>::new();
                                            let _ = write!(debug_msg, "Unimplemented packet recieved: Class 0x{:02X}, ID 0x{:02X}\r\n", class, id);
                                            self.other_msg = debug_msg.clone();
                                        },
                                };
                            }
                        }
                    }
                }
                total_to_read -= to_read;
            }
        } 
        else 
        {
            let mut debug_msg = String::<64>::new();
            let _ = write!(debug_msg, "Didn't get a message back on the i2c request\r\nStill has {} bytes\r\n", parser.buffer_len());

            self.other_msg = debug_msg;
        }

    }

    pub fn get_data_msg(&self) -> String<128>
    {
        let mut data_msg: String<128> = String::new();

        let _ = write!(data_msg,
            "Fix Type {:?}\n\rNumber of Satellites: {}\n\rLattitude: {}\n\rLongitude: {}\n\rHorizontal Accuracy: {}\n\r", 
            self.data.fix_type,self.data.num_satellites, self.data.lattitude, self.data.longitude, self.data.horizontal_accuracy);
        
        return data_msg;
    }

    pub fn get_status_msg(&self) -> String<128>
    {
        let mut status_msg: String<128> = String::new();

        let _ = write!(status_msg,
            "Status Flags: {:?}\n\rMilliseconds per GPS Week: {}\n\rNoise per Millisecond: {}\n\rUnknown Packet?: {}\n\r", 
            self.data.status_flags, self.data.millis_since_gps_week, self.data.noise_per_ms, self.data.unknown_packet);
        
        return status_msg;
    }

    pub fn get_other_msg(&self) -> String<64>
    {
        return self.other_msg.clone();
    }
}