// Run with the following command:
// cargo fuzz run fuzz_verify_cell_kzg_proof --fuzz-dir rustfuzz

#![no_main]
extern crate core;

use arbitrary::Arbitrary;
use c_kzg::KzgSettings;
use c_kzg::{Bytes48, Cell};
use eip7594::verifier::VerifierContext;
use lazy_static::lazy_static;
use libfuzzer_sys::fuzz_target;
use std::path::PathBuf;

lazy_static! {
    static ref KZG_SETTINGS: KzgSettings = {
        let root_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let trusted_setup_file = root_dir.join("..").join("src").join("trusted_setup.txt");
        KzgSettings::load_trusted_setup_file(&trusted_setup_file, 0).unwrap()
    };
    static ref VERIFIER_CONTEXT: VerifierContext = VerifierContext::new();
}

#[derive(Arbitrary, Debug)]
struct Input {
    commitment: Bytes48,
    cell_id: u64,
    cell: Cell,
    proof: Bytes48,
}

fuzz_target!(|input: Input| {
    let ckzg_result = c_kzg::KzgProof::verify_cell_kzg_proof(
        &input.commitment,
        input.cell_id,
        &input.cell,
        &input.proof,
        &KZG_SETTINGS,
    );
    let rkzg_result = VERIFIER_CONTEXT.verify_cell_kzg_proof(
        &input.commitment.as_slice(),
        input.cell_id,
        &input.cell.to_bytes().as_slice(),
        &input.proof.as_slice(),
    );

    match (&ckzg_result, &rkzg_result) {
        (Ok(ckzg_valid), Ok(())) => {
            // One returns a boolean, the other just says Ok.
            assert_eq!(*ckzg_valid, true);
        }
        (Ok(ckzg_valid), Err(err)) => {
            // If ckzg was Ok, ensure the proof was rejected.
            assert_eq!(*ckzg_valid, false);
            match err {
                eip7594::verifier::VerifierError::InvalidProof => (),
                _ => panic!("Expected InvalidProof, got {:?}", err),
            }
        }
        (Err(_), Err(_)) => {
            // Cannot compare errors, they are unique.
        }
        _ => {
            // There is a disagreement.
            panic!("mismatch {:?} and {:?}", &ckzg_result, &rkzg_result);
        }
    }
});