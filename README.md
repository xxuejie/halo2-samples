# halo2-samples

A series of [halo2](https://github.com/zcash/halo2) samples runnable in native or [Nervos CKB](https://github.com/nervosnetwork/ckb) environments.

# Usage

## How to run poseidon hash example on a Ubuntu 22.04 machine:

```bash
$ sudo apt install gcc-riscv64-unknown-elf
$ git clone https://github.com/xxuejie/halo2-samples
$ cd halo2-samples
$ cargo build --release --package poseidon_natives
$ rustup target add riscv64imac-unknown-none-elf
$ cargo build --target riscv64imac-unknown-none-elf --release --package poseidon_ckb_verifier
$ # First, use the prover to generate proofs
$ ./target/release/poseidon_native_prover abcabc ./target/riscv64imac-unknown-none-elf/release/poseidon_ckb_verifier
$ # You can tweak "abcabc" to other values you want to hash
$ # 1. Use the native verifier to verify the proofs
$ ./target/release/poseidon_native_verifier
Success!
$ # 2. Make sure you have ckb-standalone-debugger installed, you can also
$ # run the verifier in CKB environment:
$ ckb-debugger --tx-file tx.json --cell-index 0 --cell-type input --script-group-type lock --max-cycles 7000000000
Run result: 0
Total cycles consumed: 169418858(161.6M)
Transfer cycles: 53366(52.1K), running cycles: 169365492(161.5M)
$ # The cycle consumption here is still quite high, which requires more
$ # optimization work.
```
