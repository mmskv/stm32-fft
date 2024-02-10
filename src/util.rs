use embassy_stm32::pac::common;
use embassy_stm32::pac::timer;

/// metapac's functions do not work with ch5+ch6 so use these reimplementations

#[doc = "capture/compare mode register 1 (output mode)"]
#[inline(always)]
pub fn ccmr_output(ptr: *mut u8, n: usize) -> common::Reg<timer::regs::CcmrOutput, common::RW> {
    assert!(n < 3usize);
    if n == 2 {
        return unsafe { common::Reg::from_ptr(ptr.add(84usize) as _) };
    }
    unsafe { common::Reg::from_ptr(ptr.add(24usize + n * 4usize) as _) }
}

#[doc = "capture/compare register"]
#[inline(always)]
pub const fn ccr(ptr: *mut u8, n: usize) -> common::Reg<ExtCcr16, common::RW> {
    assert!(n < 6usize);
    if n == 4 {
        return unsafe { common::Reg::from_ptr(ptr.add(88usize) as _) };
    }
    return unsafe { common::Reg::from_ptr(ptr.add(52usize + n * 4usize) as _) };
}

#[doc = "capture/compare register 1"]
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ExtCcr16(pub u32);
impl ExtCcr16 {
    #[doc = "Capture/Compare 1 value"]
    #[inline(always)]
    pub const fn ccr(&self) -> u16 {
        let val = (self.0 >> 0usize) & 0xffff;
        val as u16
    }
    #[doc = "Capture/Compare 1 value"]
    #[inline(always)]
    pub fn set_ccr(&mut self, val: u16) {
        self.0 = (self.0 & !(0xffff << 0usize)) | (((val as u32) & 0xffff) << 0usize);
    }
    #[doc = "Capture/Compare 1 value"]
    #[inline(always)]
    pub fn set_ccr_ch5(&mut self, val: u16) {
        self.0 = (self.0 & !(0xffffffff)) | ((val as u32) & 0xffffffff);
    }
    #[doc = "Capture/Compare 1 value"]
    #[inline(always)]
    pub fn set_ccr_group_ch5_ch1(&mut self) {
        let gc5c1 = 1 << 29;
        // self.0 &= !gc5c3; // clear bit fields
        self.0 = self.0 | gc5c1;
    }
}
impl Default for ExtCcr16 {
    #[inline(always)]
    fn default() -> ExtCcr16 {
        ExtCcr16(0)
    }
}

#[doc = "capture/compare enable register"]
#[inline(always)]
pub const fn ccer(ptr: *mut u8) -> common::Reg<ExtCcerAdv, common::RW> {
    unsafe { common::Reg::from_ptr(ptr.add(32usize) as _) }
}

#[doc = "capture/compare enable register"]
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ExtCcerAdv(pub u32);
impl ExtCcerAdv {
    #[doc = "Capture/Compare 1 output enable"]
    #[inline(always)]
    pub const fn cce(&self, n: usize) -> bool {
        assert!(n < 6usize);
        let offs = 0usize + n * 4usize;
        let val = (self.0 >> offs) & 0x01;
        val != 0
    }
    #[doc = "Capture/Compare 1 output enable"]
    #[inline(always)]
    pub fn set_cce(&mut self, n: usize, val: bool) {
        assert!(n < 6usize);
        let offs = 0usize + n * 4usize;
        self.0 = (self.0 & !(0x01 << offs)) | (((val as u32) & 0x01) << offs);
    }
    #[doc = "Capture/Compare 1 output Polarity"]
    #[inline(always)]
    pub const fn ccp(&self, n: usize) -> bool {
        assert!(n < 6usize);
        let offs = 1usize + n * 4usize;
        let val = (self.0 >> offs) & 0x01;
        val != 0
    }
    #[doc = "Capture/Compare 1 output Polarity"]
    #[inline(always)]
    pub fn set_ccp(&mut self, n: usize, val: bool) {
        assert!(n < 6usize);
        let offs = 1usize + n * 4usize;
        self.0 = (self.0 & !(0x01 << offs)) | (((val as u32) & 0x01) << offs);
    }
    #[doc = "Capture/Compare 1 complementary output enable"]
    #[inline(always)]
    pub const fn ccne(&self, n: usize) -> bool {
        assert!(n < 6usize);
        let offs = 2usize + n * 4usize;
        let val = (self.0 >> offs) & 0x01;
        val != 0
    }
    #[doc = "Capture/Compare 1 complementary output enable"]
    #[inline(always)]
    pub fn set_ccne(&mut self, n: usize, val: bool) {
        assert!(n < 6usize);
        let offs = 2usize + n * 4usize;
        self.0 = (self.0 & !(0x01 << offs)) | (((val as u32) & 0x01) << offs);
    }
    #[doc = "Capture/Compare 1 output Polarity"]
    #[inline(always)]
    pub const fn ccnp(&self, n: usize) -> bool {
        assert!(n < 6usize);
        let offs = 3usize + n * 4usize;
        let val = (self.0 >> offs) & 0x01;
        val != 0
    }
    #[doc = "Capture/Compare 1 output Polarity"]
    #[inline(always)]
    pub fn set_ccnp(&mut self, n: usize, val: bool) {
        assert!(n < 6usize);
        let offs = 3usize + n * 4usize;
        self.0 = (self.0 & !(0x01 << offs)) | (((val as u32) & 0x01) << offs);
    }
}
impl Default for ExtCcerAdv {
    #[inline(always)]
    fn default() -> ExtCcerAdv {
        ExtCcerAdv(0)
    }
}
