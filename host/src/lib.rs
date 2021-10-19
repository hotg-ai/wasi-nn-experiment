use crate::types::NnErrno;
use wiggle::GuestErrorType;

wiggle::from_witx!({
    witx: ["$CARGO_MANIFEST_DIR/../vendor/wasi-nn/phases/ephemeral/witx/wasi_ephemeral_nn.witx"]
});

impl GuestErrorType for NnErrno {
    fn success() -> Self { NnErrno::Success }
}
