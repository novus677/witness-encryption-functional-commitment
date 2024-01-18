use ark_bls12_381::Fr as ScalarField;
use ark_bls12_381::{Bls12_381, Fq12, G1Projective, G2Projective};
use ark_ec::pairing::{Pairing, PairingOutput};
use ark_ec::Group;
use ark_ff::Field;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, SerializationError};
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::encrypt;
use crate::linear_fc;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn copy_vec_to_u8arr(v: &Vec<u8>) -> Uint8Array {
    let u8_arr = Uint8Array::new_with_length(v.len() as u32);
    u8_arr.copy_from(v);
    u8_arr
}

#[derive(Serialize, Deserialize)]
struct Commitment {
    commit: Vec<u8>,
    r_commit: Vec<u8>,
}

#[wasm_bindgen]
pub fn commit(ckey_bytes_1: &[u8], ckey_bytes_2: &[u8], x: u32) -> JsValue {
    let u1 = G1Projective::deserialize_compressed(ckey_bytes_1)
        .expect("deserialization should not fail");
    let u2 = G2Projective::deserialize_compressed(ckey_bytes_2)
        .expect("deserialization should not fail");
    let ckey = linear_fc::CommitmentKey {
        u1: vec![u1],
        u2: vec![u2],
    };

    let x = ScalarField::from(x);
    let (commit, r_commit) = linear_fc::commit(&ckey, &vec![x]);
    let mut commit_serial: Vec<u8> = vec![];
    commit
        .serialize_compressed(&mut commit_serial)
        .expect("serialization should not fail");
    let mut r_commit_serial: Vec<u8> = vec![];
    r_commit
        .serialize_compressed(&mut r_commit_serial)
        .expect("serialization should not fail");

    serde_wasm_bindgen::to_value(&Commitment {
        commit: commit_serial,
        r_commit: r_commit_serial,
    })
    .unwrap()
}

#[derive(Serialize, Deserialize)]
struct Ciphertext {
    proj_key_bytes: Vec<u8>,
    rand_bytes: Vec<u8>,
    ciphertext: u8,
}

#[wasm_bindgen]
pub fn encrypt(
    ckey_bytes_1: &[u8],
    ckey_bytes_2: &[u8],
    commit: &[u8],
    y: u32,
    message: u8,
) -> JsValue {
    let u1 = G1Projective::deserialize_compressed(ckey_bytes_1)
        .expect("deserialization should not fail");
    let u2 = G2Projective::deserialize_compressed(ckey_bytes_2)
        .expect("deserialization should not fail");
    let ckey = linear_fc::CommitmentKey {
        u1: vec![u1],
        u2: vec![u2],
    };

    let commit =
        G1Projective::deserialize_compressed(commit).expect("deserialization should not fail");
    let y = ScalarField::from(y);
    let ct_raw = encrypt::encrypt(&ckey, &commit, &vec![ScalarField::from(1)], y, message).unwrap();

    let mut proj_key_serial: Vec<u8> = vec![];
    ct_raw
        .proj_key
        .serialize_compressed(&mut proj_key_serial)
        .expect("serialization should not fail");
    let ct_final = Ciphertext {
        proj_key_bytes: proj_key_serial,
        rand_bytes: ct_raw.rand_bytes,
        ciphertext: ct_raw.ciphertext,
    };

    serde_wasm_bindgen::to_value(&ct_final).unwrap()
}

#[wasm_bindgen]
pub fn decrypt(
    ckey_bytes_1: &[u8],
    ckey_bytes_2: &[u8],
    proj_key_bytes: &[u8],
    rand_bytes: &[u8],
    ciphertext: u8,
    x: u32,
    r_commit_bytes: &[u8],
) -> u8 {
    let u1 = G1Projective::deserialize_compressed(ckey_bytes_1)
        .expect("deserialization should not fail");
    let u2 = G2Projective::deserialize_compressed(ckey_bytes_2)
        .expect("deserialization should not fail");
    let ckey = linear_fc::CommitmentKey {
        u1: vec![u1],
        u2: vec![u2],
    };

    let proj_key = G2Projective::deserialize_compressed(proj_key_bytes)
        .expect("deserialization should not fail");
    let ct_raw = encrypt::Ciphertext {
        proj_key,
        rand_bytes: rand_bytes.to_vec(),
        ciphertext,
    };

    let r_commit = ScalarField::deserialize_compressed(r_commit_bytes)
        .expect("deserialization should not fail");
    let opening = linear_fc::open(
        &ckey,
        &vec![ScalarField::from(x)],
        r_commit,
        &vec![ScalarField::from(1)],
    );

    encrypt::decrypt(&ckey, &ct_raw, &opening).unwrap()
}
