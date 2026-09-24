// TOKIO_PROOF: verus-backend-smoke
// Infrastructure smoke proof: this checks that the configured Verus backend
// can establish and discharge a simple pre/postcondition contract.

use vstd::prelude::*;

verus! {
fn successor(x: u8) -> (y: u8)
    requires
        x < 255,
    ensures
        y == x + 1,
{
    x + 1
}
}

fn main() {}
