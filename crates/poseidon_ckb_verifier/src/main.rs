#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

use alloc::format;
use ckb_std::{
    ckb_constants::Source,
    default_alloc,
    syscalls::{debug, load_witness},
};
use core::arch::asm;

ckb_std::entry!(program_entry);
default_alloc!();

use halo2_gadgets::poseidon::primitives::Spec;
use halo2_proofs::{
    pasta::{vesta, Fp},
    plonk::{verify_proof, SingleVerifier, VerifyingKey},
    poly::commitment::Params,
    transcript::{Blake2bRead, Challenge255},
};
use poseidon_ckb_verifier::MySpec;

use alloc::string::{String, ToString};

fn verify_poseidon<S, const WIDTH: usize, const RATE: usize, const L: usize>(
    proof: &[u8],
    output_data: &[u8],
    params_data: &[u8],
    vk_data: &[u8],
) -> Result<(), String>
where
    S: Spec<Fp, WIDTH, RATE> + Copy + Clone,
{
    // Initialize the polynomial commitment parameters
    let params: Params<vesta::Affine> =
        Params::read(&mut &params_data[..]).map_err(|e| e.to_string())?;

    // Initialize the verifying key
    let vk: VerifyingKey<vesta::Affine> =
        VerifyingKey::read(&mut &vk_data[..]).map_err(|e| e.to_string())?;

    if output_data.len() != 32 {
        return Err(format!("Invalid output data length: {}", output_data.len()));
    }
    let output = {
        let mut raw = [0u64; 4];
        for i in 0..32 {
            raw[i / 8] |= (output_data[i] as u64) << ((i % 8) * 8);
        }
        Fp::from_raw(raw)
    };

    let strategy = SingleVerifier::new(&params);
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
    verify_proof(&params, &vk, strategy, &[&[&[output]]], &mut transcript)
        .map_err(|e| e.to_string())
}

pub fn program_entry(_argc: u64, _argv: *const *const u8) -> i8 {
    let mut params_buffer = [0u8; 32 * 1024];
    let params_length = match load_witness(&mut params_buffer, 0, 0, Source::Input) {
        Ok(l) => l,
        Err(e) => {
            debug(format!("Loading params error {:?}", e));
            return -1;
        }
    };
    let mut vk_buffer = [0u8; 32 * 1024];
    let vk_length = match load_witness(&mut vk_buffer, 0, 1, Source::Input) {
        Ok(l) => l,
        Err(e) => {
            debug(format!("Loading vk error {:?}", e));
            return -1;
        }
    };
    let mut proof_buffer = [0u8; 32 * 1024];
    let proof_length = match load_witness(&mut proof_buffer, 0, 2, Source::Input) {
        Ok(l) => l,
        Err(e) => {
            debug(format!("Loading proof error {:?}", e));
            return -1;
        }
    };
    let mut output_buffer = [0u8; 32];
    let output_length = match load_witness(&mut output_buffer, 0, 3, Source::Input) {
        Ok(l) => l,
        Err(e) => {
            debug(format!("Loading output error {:?}", e));
            return -1;
        }
    };
    if let Err(e) = verify_poseidon::<MySpec<3, 2>, 3, 2, 2>(
        &proof_buffer[0..proof_length],
        &output_buffer[0..output_length],
        &params_buffer[0..params_length],
        &vk_buffer[0..vk_length],
    ) {
        debug(format!("Verification error: {:?}", e));
        return -1;
    }
    0
}
