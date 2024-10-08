use super::framework::TestFramework;
use crate::circuits::{
    polynomials::generic::testing::{create_circuit, fill_in_witness},
    wires::COLUMNS,
};
use ark_ff::Zero;
use mina_curves::pasta::{Fp, Vesta, VestaParameters};
use mina_poseidon::{
    constants::PlonkSpongeConstantsKimchi,
    sponge::{DefaultFqSponge, DefaultFrSponge},
};
use std::array;

type SpongeParams = PlonkSpongeConstantsKimchi;
type BaseSponge = DefaultFqSponge<VestaParameters, SpongeParams>;
type ScalarSponge = DefaultFrSponge<Fp, SpongeParams>;

#[test]
fn test_generic_gate() {
    let gates = create_circuit(0, 0);

    // create witness
    let mut witness: [Vec<Fp>; COLUMNS] = array::from_fn(|_| vec![Fp::zero(); gates.len()]);
    fill_in_witness(0, &mut witness, &[]);

    // create and verify proof based on the witness
    TestFramework::<Vesta>::default()
        .gates(gates)
        .witness(witness)
        .setup()
        .prove_and_verify::<BaseSponge, ScalarSponge>()
        .unwrap();
}

#[test]
fn test_generic_gate_pub() {
    let public = vec![Fp::from(3u8); 5];
    let gates = create_circuit(0, public.len());

    // create witness
    let mut witness: [Vec<Fp>; COLUMNS] = array::from_fn(|_| vec![Fp::zero(); gates.len()]);
    fill_in_witness(0, &mut witness, &public);

    // create and verify proof based on the witness
    TestFramework::<Vesta>::default()
        .gates(gates)
        .witness(witness)
        .public_inputs(public)
        .setup()
        .prove_and_verify::<BaseSponge, ScalarSponge>()
        .unwrap();
}

#[test]
fn test_generic_gate_pub_all_zeros() {
    let public = vec![Fp::from(0u8); 5];
    let gates = create_circuit(0, public.len());

    // create witness
    let mut witness: [Vec<Fp>; COLUMNS] = array::from_fn(|_| vec![Fp::zero(); gates.len()]);
    fill_in_witness(0, &mut witness, &public);

    // create and verify proof based on the witness
    TestFramework::<Vesta>::default()
        .gates(gates)
        .witness(witness)
        .public_inputs(public)
        .setup()
        .prove_and_verify::<BaseSponge, ScalarSponge>()
        .unwrap();
}

#[test]
fn test_generic_gate_pub_empty() {
    let public = vec![];
    let gates = create_circuit(0, public.len());

    // create witness
    let mut witness: [Vec<Fp>; COLUMNS] = array::from_fn(|_| vec![Fp::zero(); gates.len()]);
    fill_in_witness(0, &mut witness, &public);

    // create and verify proof based on the witness
    TestFramework::<Vesta>::default()
        .gates(gates)
        .witness(witness)
        .public_inputs(public)
        .setup()
        .prove_and_verify::<BaseSponge, ScalarSponge>()
        .unwrap();
}

#[cfg(feature = "bn254")]
#[test]
fn test_generic_gate_pairing() {
    type Fp = ark_bn254::Fr;
    type SpongeParams = PlonkSpongeConstantsKimchi;
    type BaseSponge = DefaultFqSponge<ark_bn254::g1::Config, SpongeParams>;
    type ScalarSponge = DefaultFrSponge<Fp, SpongeParams>;

    use ark_ff::UniformRand;

    let public = vec![Fp::from(3u8); 5];
    let gates = create_circuit(0, public.len());

    let rng = &mut rand::rngs::OsRng;
    let x = Fp::rand(rng);

    // create witness
    let mut witness: [Vec<Fp>; COLUMNS] = array::from_fn(|_| vec![Fp::zero(); gates.len()]);
    fill_in_witness(0, &mut witness, &public);

    // create and verify proof based on the witness
    <TestFramework<
        _,
        poly_commitment::pairing_proof::PairingProof<ark_ec::bn::Bn<ark_bn254::Config>>,
    > as Default>::default()
    .gates(gates)
    .witness(witness)
    .public_inputs(public)
    .setup_with_custom_srs(|d1, usize| {
        let srs = poly_commitment::pairing_proof::PairingSRS::create(x, usize);
        srs.full_srs.get_lagrange_basis(d1);
        srs
    })
    .prove_and_verify::<BaseSponge, ScalarSponge>()
    .unwrap();
}
