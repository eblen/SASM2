#[derive(Debug)]
pub enum Zpm {
    Apple { bytes_remaining: u16 },
    Atari2600 { next_free_byte: u16 },
}

impl Zpm {
    // Allow creating specific variants without needing to check for failure
    pub fn new_for_apple() -> Self {
        Self::new("apple").expect("Internal error: Unable to create an AppleII ZPM")
    }

    pub fn new_for_atari() -> Self {
        Self::new("atari").expect("Internal error: Unable to create an Atari 2600 ZPM")
    }

    // Attempt to create a variant from a string
    pub fn new(arch: &str) -> Result<Self, &str> {
        if arch.to_ascii_lowercase().starts_with("apple") {
            return Ok(Zpm::Apple {
                bytes_remaining: 0x100,
            });
        }

        if arch.to_ascii_lowercase().starts_with("atari") {
            return Ok(Zpm::Atari2600 {
                next_free_byte: 0x80,
            });
        }

        Err("Unrecognized or unsupported system")
    }

    pub fn alloc(&mut self, size: u16) -> u8 {
        match self {
            // Apple II system-level programs, like the monitor and DOS, use the
            // lower addresses first and leave the higher addresses for user
            // programs. Thus, this simple manager allocates bytes in order from
            // high to low memory. A program that uses lots of zero-page bytes
            // will need a more sophisticated manager. It also will have to
            // consider the specific Apple II model being used.
            Zpm::Apple { bytes_remaining: b } => {
                if size == 0 {
                    panic!("Request to allocate zero bytes of zero page memory");
                }

                if size > *b {
                    panic!("Zero page memory exhausted");
                }

                *b -= size;
                return *b as u8;
            }

            // The upper half of zero page (0x80 - 0xff) is the ONLY memory,
            // zero-page or otherwise, that Atari 2600 programmers have
            // available. Furthermore, the stack is mapped to zero page as well!
            // The stack normally starts at ff and grows down, which means that
            // the lower addresses should be preferred. Accordingly, this
            // manager allocates memory in order from 0x80 to 0xff.
            Zpm::Atari2600 { next_free_byte: b } => {
                if size == 0 {
                    panic!("Request to allocate zero bytes of zero page memory");
                }

                if *b + size > 0x100 {
                    panic!("Zero page memory exhausted");
                }

                *b += size;
                return (*b - size) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Request to allocate zero bytes of zero page memory")]
    fn zpm_alloc_0_apple() {
        Zpm::new_for_apple().alloc(0);
    }

    #[test]
    #[should_panic(expected = "Request to allocate zero bytes of zero page memory")]
    fn zpm_alloc_0_atari() {
        Zpm::new_for_atari().alloc(0);
    }

    #[test]
    #[should_panic(expected = "Zero page memory exhausted")]
    fn zpm_alloc_too_much_apple() {
        let mut zpm = Zpm::new_for_apple();
        zpm.alloc(100);
        zpm.alloc(100);
        zpm.alloc(57);
    }

    #[test]
    #[should_panic(expected = "Zero page memory exhausted")]
    fn zpm_alloc_too_much_atari() {
        let mut zpm = Zpm::new_for_atari();
        zpm.alloc(50);
        zpm.alloc(50);
        zpm.alloc(29);
    }

    #[test]
    fn zpm_alloc_all_available_apple() {
        let mut zpm = Zpm::new_for_apple();
        let addr1 = zpm.alloc(100);
        let addr2 = zpm.alloc(100);
        let addr3 = zpm.alloc(56);
        assert!(addr1 == 0xff - 99 && addr2 == 0xff - 199 && addr3 == 0);
    }

    #[test]
    fn zpm_alloc_all_available_atari() {
        let mut zpm = Zpm::new_for_atari();
        let addr1 = zpm.alloc(50);
        let addr2 = zpm.alloc(50);
        let addr3 = zpm.alloc(28);
        assert!(addr1 == 0x80 && addr2 == 0x80 + 50 && addr3 == 0x80 + 100);
    }
}
