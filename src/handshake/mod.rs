mod state;
mod c0c1;
mod s0s1s2;

pub use state::*;
pub use c0c1::*;
pub use s0s1s2::*;

use crate::{Error, Result};

pub fn validate_c0c1(data: &[u8]) -> Result<C0C1> {
    // Parse C0+C1
    let c0c1 = C0C1::parse(data)?;

    // Detect format
    let format = c0c1.detect_format();

    // Validate based on format
    c0c1.validate_digest(format)?;

    Ok(c0c1)
}

pub fn generate_s0s1s2(c0c1: &C0C1) -> Result<Vec<u8>> {
    // Detect client handshake format
    let format = c0c1.detect_format();

    // Generate appropriate response
    let s0s1s2 = if format == HandshakeFormat::Simple {
        S0S1S2::generate(c0c1)?
    } else {
        S0S1S2::generate_complex(c0c1, format)?
    };

    Ok(s0s1s2.encode())
}

pub fn validate_c2(c2_data: &[u8], s0s1s2: &S0S1S2) -> Result<()> {
    // Parse C2
    let c2 = C2::parse(c2_data)?;

    // Validate against S1
    c2.validate(s0s1s2)?;

    Ok(())
}