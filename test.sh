set -o errexit
set -o nounset
set -o pipefail

export RUST_BACKTRACE=1
cargo build
TESTS=(sr/SPIRTREV.MZX btb/BERNARD.MZX caverns/CAVERNS.mzx DE_Game/DE_MAIN.MZX rd3TSE/rd3TSE.mzx insanifs/INSANIFS.MZX 30641/30641.mzx srpgmain/SMILYRPG.MZX chronos/CHRONOS.MZX forest/FOREST.MZX catacomb/CATACOMB.MZX weird1/WEIRD1.MZX weirdse1/WEIRDSE1.MZX)
for test in ${TESTS[@]}; do
    echo Testing $test.
    target/debug/mzxview ~/mzx/${test} 0 out.png
    echo Passed.
done
