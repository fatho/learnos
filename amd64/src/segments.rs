

/// A segment selector
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct Selector(pub u16);

impl Selector {
    pub const NULL: Selector = Selector(0);
}

/// Privilege level
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct Ring(u8);

impl Ring {
    pub const RING0: Ring = Ring(0);
    pub const RING1: Ring = Ring(1);
    pub const RING2: Ring = Ring(2);
    pub const RING3: Ring = Ring(3);

    /// Create a new ring if the number is valid (i.e. in the range 0 (kernel mode) - 3 (user mode))
    pub fn new(ring: u8) -> Option<Ring> {
        if ring <= 3 {
            Some(Ring(ring))
        } else {
            None
        }
    }

    pub fn number(&self) -> u8 {
        self.0
    }
}
