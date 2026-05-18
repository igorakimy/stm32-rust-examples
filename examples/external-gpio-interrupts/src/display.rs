use embedded_hal::digital::OutputPin;

pub struct SevenSegmentDisplay<A, B, C, D, E, F, G, DP> {
    pub a: A,
    pub b: B,
    pub c: C,
    pub d: D,
    pub e: E,
    pub f: F,
    pub g: G,
    pub dp: DP,
}

impl<A, B, C, D, E, F, G, DP> SevenSegmentDisplay<A, B, C, D, E, F, G, DP>
where
    A: OutputPin,
    B: OutputPin,
    C: OutputPin,
    D: OutputPin,
    E: OutputPin,
    F: OutputPin,
    G: OutputPin,
    DP: OutputPin,
{
    pub fn clear(&mut self) {
        self.a.set_low().ok();
        self.b.set_low().ok();
        self.c.set_low().ok();
        self.d.set_low().ok();
        self.e.set_low().ok();
        self.f.set_low().ok();
        self.g.set_low().ok();
        self.dp.set_low().ok();
    }

    pub fn show(&mut self, digit: u8) {
        self.clear();
        match digit {
            0 => {
                self.a.set_high().ok();
                self.b.set_high().ok();
                self.c.set_high().ok();
                self.d.set_high().ok();
                self.e.set_high().ok();
                self.f.set_high().ok();
            }
            1 => {
                self.b.set_high().ok();
                self.c.set_high().ok();
            }
            2 => {
                self.a.set_high().ok();
                self.b.set_high().ok();
                self.g.set_high().ok();
                self.e.set_high().ok();
                self.d.set_high().ok();
            }
            3 => {
                self.a.set_high().ok();
                self.b.set_high().ok();
                self.c.set_high().ok();
                self.d.set_high().ok();
                self.g.set_high().ok();
            }
            4 => {
                self.b.set_high().ok();
                self.c.set_high().ok();
                self.f.set_high().ok();
                self.g.set_high().ok();
            }
            5 => {
                self.a.set_high().ok();
                self.c.set_high().ok();
                self.d.set_high().ok();
                self.f.set_high().ok();
                self.g.set_high().ok();
            }
            6 => {
                self.a.set_high().ok();
                self.c.set_high().ok();
                self.d.set_high().ok();
                self.e.set_high().ok();
                self.f.set_high().ok();
                self.g.set_high().ok();
            }
            7 => {
                self.a.set_high().ok();
                self.b.set_high().ok();
                self.c.set_high().ok();
            }
            8 => {
                self.a.set_high().ok();
                self.b.set_high().ok();
                self.c.set_high().ok();
                self.d.set_high().ok();
                self.e.set_high().ok();
                self.f.set_high().ok();
                self.g.set_high().ok();
            }
            9 => {
                self.a.set_high().ok();
                self.b.set_high().ok();
                self.c.set_high().ok();
                self.d.set_high().ok();
                self.f.set_high().ok();
                self.g.set_high().ok();
            }
            _ => ()
        }
    }
}
