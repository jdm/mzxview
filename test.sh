set -o errexit
set -o nounset
set -o pipefail

export RUST_BACKTRACE=1
cargo run ~/mzx/sr/SPIRTREV.MZX 0 out.png
cargo run ~/mzx/btb/BERNARD.MZX 0 out.png
cargo run ~/mzx/caverns/CAVERNS.MZX 0 out.png
