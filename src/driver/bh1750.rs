use embedded_hal::i2c::{I2c, Operation, SevenBitAddress};
use esp_idf_hal::delay::Delay;

const BH1750_ADDRESS: u8 = 0x23;
const MEASURMENT_ACCURACY: f64 = 1.2;

pub struct BH1750<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> BH1750<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C) -> Result<Self, E> {
        let mut bh1750 = Self {
            i2c,
            address: BH1750_ADDRESS,
        };

        bh1750.init()?;

        Ok(bh1750)
    }

    fn init(&mut self) -> Result<(), E> {
        // Continuously H-Resolution Mode
        let payload = &[0x01];
        self.i2c.write(self.address, payload)?;
        Ok(())
    }

    pub fn read_lux(&mut self, delay: &Delay) -> Result<f64, E> {
        let mut data = [0u8; 2];

        delay.delay_ms(180);

        self.i2c.write_read(self.address, &[0x00], &mut data)?;
        let raw = u16::from_be_bytes(data);
        Ok((raw as f64) / MEASURMENT_ACCURACY)
    }
}
