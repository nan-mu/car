/// 包含金属探测的初始化和驱动
use rppal::i2c::{self, I2c, Result};

#[derive(Debug)]
pub enum LdcErr {
    Bus,
    ConversionUnderRange,
    ConversionOverRange,
    ConversionWatchdogTimeout,
    ConversionAmplitude,
}

/// TI LDC1x1x driver instance
pub struct Ldc {
    bus: I2c,
}

impl Ldc {
    pub fn new(bus: I2c) -> Self {
        Ldc { bus }
    }

    pub fn write_reg(&mut self, reg: u8, data: u16) -> Result<()> {
        self.bus.block_write(reg, &[(data >> 8) as u8, data as u8])
    }

    pub fn read_reg(&mut self, reg: u8) -> Result<u16> {
        let mut result: [u8; 2] = [0xde, 0xad];
        self.bus.block_read(reg, &mut result).unwrap();
        Ok((result[0] as u16) << 8 | result[1] as u16)
    }

    /// Read the conversion result for a channel.
    /// Error flags from the result are returned as errors.
    /// Reading does clear the error flags on the device.
    ///
    /// This function must only be used with 12-bit devices (LDC131x).
    /// Use read_data_24bit with 24-bit devices (LDC161x).
    pub fn read_data_12bit(&mut self, ch: Channel) -> Result<u16> {
        let b = self.read_reg(2 * ch as u8)?;
        if b & (1 << 15) != 0 {
            return Err(i2c::Error::UnknownModel);
        }
        if b & (1 << 14) != 0 {
            return Err(i2c::Error::UnknownModel);
        }
        if b & (1 << 13) != 0 {
            return Err(i2c::Error::UnknownModel);
        }
        if b & (1 << 12) != 0 {
            return Err(i2c::Error::UnknownModel);
        }
        Ok(b & 0x0fff)
    }

    /// Read the conversion result for a channel.
    /// Error flags from the result are returned as errors.
    /// Reading does clear the error flags on the device.
    ///
    /// This function must only be used with 24-bit devices (LDC161x).
    /// Use read_data_12bit with 12-bit devices (LDC131x).
    pub fn read_data_24bit(&mut self, ch: Channel) -> Result<u32> {
        Ok((self.read_data_12bit(ch)? as u32) << 16 | self.read_reg(1 + 2 * ch as u8)? as u32)
    }

    pub fn set_ref_count_conv_interval(&mut self, ch: Channel, intv: u16) -> Result<()> {
        self.write_reg(0x08 + ch as u8, intv)
    }

    pub fn set_conv_settling_time(&mut self, ch: Channel, cnt: u16) -> Result<()> {
        self.write_reg(0x10 + ch as u8, cnt)
    }

    pub fn set_clock_dividers(&mut self, ch: Channel, divs: ClockDividers) -> Result<()> {
        self.write_reg(0x14 + ch as u8, divs.fin_div << 12 | divs.fref_div)
    }

    pub fn set_error_config(&mut self, conf: ErrorConfig) -> Result<()> {
        self.write_reg(0x19, conf.0)
    }

    pub fn set_config(&mut self, conf: Config) -> Result<()> {
        self.write_reg(0x1A, conf.0)
    }

    pub fn set_mux_config(&mut self, conf: MuxConfig) -> Result<()> {
        self.write_reg(0x1B, conf.0)
    }

    pub fn reset(&mut self) -> Result<()> {
        self.write_reg(0x1C, 1 << 15)
    }

    // TODO: 131x also have a gain field in the reset register

    pub fn set_sensor_drive_current(&mut self, ch: Channel, cur: u8) -> Result<()> {
        self.write_reg(0x1E + ch as u8, (cur as u16) << 11)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ClockDividers {
    pub fin_div: u16,
    pub fref_div: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct Fsensor(pub f32);

impl Fsensor {
    /// Calculate sensor frequency based on inductance in μH and capacitance in pF.
    pub fn from_inductance_capacitance(inductance: f32, capacitance: f32) -> Self {
        Self(
            1.0 / (2.0 * 3.14 * (inductance * 1e-6_f32 * capacitance * 1e-12_f32)).sqrt()
                * 1e-6_f32,
        )
    }

    /// Calculate minimum clock dividers based on Fsensor and Fref (oscillator frequency in MHz).
    ///
    /// If using the internal oscillator, you can pass None, it will default to 43 MHz.
    pub fn to_clock_dividers(&self, ext_clk_freq: Option<f32>) -> ClockDividers {
        // unwrap_or is not const fn?!
        let fref = match ext_clk_freq {
            None => 43.0, // internal oscillator
            Some(x) => x,
        };
        ClockDividers {
            fin_div: (self.0 / 8.75 + 1.0) as u16,
            fref_div: if self.0 * 4.0 < fref {
                1
            } else if self.0 / 2.0 * 4.0 < fref {
                2
            } else {
                4
            },
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Channel {
    Zero = 0,
    One,
}

#[derive(Debug, Clone, Copy)]
pub struct Status(pub u16);

#[derive(Debug, Default, Clone, Copy)]
pub struct ErrorConfig(pub u16);

impl ErrorConfig {
    #[inline(always)]
    pub const fn with_amplitude_high_error_to_data_register(self, val: bool) -> Self {
        Self(self.0 & !(1 << 12) | (val as u16) << 12)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Config(pub u16);

impl Default for Config {
    fn default() -> Self {
        Self(
            0x1001, /* ch0, only Rp override, reserved first bit 1 */
        )
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Deglitch {
    ThreePointThreeMHz = 0b100,
}

#[derive(Debug, Clone, Copy)]
pub struct MuxConfig(pub u16);

impl Default for MuxConfig {
    fn default() -> Self {
        Self(
            0b0100_0001_111, /* reserved + default 33 MHz deglitch */
        )
    }
}

impl MuxConfig {
    #[inline(always)]
    pub const fn with_auto_scan(self, val: bool) -> Self {
        Self(self.0 & !(1 << 15) | (val as u16) << 15)
    }

    #[inline(always)]
    pub const fn with_deglitch_filter_bandwidth(self, bw: Deglitch) -> Self {
        Self(self.0 & !0b111 | bw as u16)
    }
}
