set -o errexit
set -o nounset
set -o pipefail

export RUST_BACKTRACE=1
cargo run ~/mzx/sr/SPIRTREV.MZX 0 out.png
cargo run ~/mzx/btb/BERNARD.MZX 0 out.png
cargo run ~/mzx/caverns/CAVERNS.MZX 0 out.png
cargo run ~/mzx/DE_Game/DE_MAIN.MZX 0 out.png
cargo run ~/mzx/rd3TSE/rd3TSE.mzx 0 out.png
cargo run ~/mzx/insanifs/INSANIFS.mzx 0 out.png
