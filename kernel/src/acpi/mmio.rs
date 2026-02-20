
pub struct PM1Status(pub u16);
impl PM1Status { 
    const TMR_STS: u16         = (1 << 0);
    const BM_STS: u16          = (1 << 4);
    const GBL_STS: u16         = (1 << 5);
    const PWRBTN_STS: u16      = (1 << 8);
    const SLPBTN_STS: u16      = (1 << 9);
    const RTC_STS: u16         = (1 << 10);
    const PICEXP_WAKE_STS: u16 = (1 << 14);
    const WAK_STS: u16         = (1 << 15);
}
impl Into<u16> for PM1Status { 
    fn into(self) -> u16 { self.0 }
}

pub struct PM1Enable(pub u16);
impl PM1Enable { 
    const TMR_EN: u16          = (1 << 0);
    const GBL_EN: u16          = (1 << 5);
    const PWRBTN_EN: u16       = (1 << 8);
    const SLPBTN_EN: u16       = (1 << 9);
    const RTC_EN: u16          = (1 << 10);
    const PICEXP_WAKE_DIS: u16 = (1 << 14);
}
impl Into<u16> for PM1Enable { 
    fn into(self) -> u16 { self.0 }
}


pub struct PM1Control(pub u16);
impl PM1Enable { 
    const SCI_EN: u16   = (1 << 0);
    const BM_RLD: u16   = (1 << 1);
    const GBL_RLS: u16  = (1 << 2);
    const SLP_TYPx: u16 = (0b111 << 10);
    const SLP_EN: u16   = (1 << 13);
}
impl Into<u16> for PM1Control { 
    fn into(self) -> u16 { self.0 }
}


