# halo2-samples

A series of [halo2](https://github.com/zcash/halo2) samples runnable in native or [Nervos CKB](https://github.com/nervosnetwork/ckb) environments.

# Usage

## How to run poseidon hash example on a Ubuntu 22.04 machine:

```bash
$ sudo apt install gcc-riscv64-unknown-elf
$ git clone https://github.com/xxuejie/halo2-samples
$ cd halo2-samples
$ git clone https://github.com/nervosnetwork/ckb-vm
$ # optionally, change ckb-vm to the revision you want to test against
$ cargo build --release --package poseidon_natives --package halo2-runner
$ rustup target add riscv64imac-unknown-none-elf
$ cargo build --target riscv64imac-unknown-none-elf --release --package poseidon_ckb_verifier
$ # First, use the prover to generate proofs
$ ./target/release/poseidon_native_prover abcabc ./target/riscv64imac-unknown-none-elf/release/poseidon_ckb_verifier
$ # You can tweak "abcabc" to other values you want to hash
$ # 1. Use the native verifier to verify the proofs
$ ./target/release/poseidon_native_verifier
Success!
$ # 2. If you have ckb-standalone-debugger installed, you can
$ # run the verifier in CKB environment:
$ ckb-debugger --tx-file tx.json --cell-index 0 --cell-type input --script-group-type lock --max-cycles 7000000000
Run result: 0
Total cycles consumed: 77527680(73.9M)
Transfer cycles: 74526(72.8K), running cycles: 77453154(73.9M)
$ # 3. Or you can use the halo2-runner included in this repo, which can be
$ # compiled using newer assembly based ckb-vm:
$ ./target/release/halo2-runner --tx-file tx.json --cell-index 0 --cell-type input --script-type lock
Cycles: 77527680
Runtime: 136.015218ms
```
