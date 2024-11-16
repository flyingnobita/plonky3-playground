use p3_baby_bear::{BabyBear, Poseidon2BabyBear};
use p3_challenger::DuplexChallenger;
use p3_commit::ExtensionMmcs;
use p3_dft::Radix2DitParallel;
use p3_field::extension::BinomialExtensionField;
use p3_field::Field;
use p3_fri::{FriConfig, TwoAdicFriPcs};
use p3_merkle_tree::MerkleTreeMmcs;
use p3_symmetric::{PaddingFreeSponge, TruncatedPermutation};
use p3_uni_stark::{prove, verify, StarkConfig};
use rand::thread_rng;

use crate::{generate_fibonacci_trace, FibonacciAir};

// Your choice of Field
type Val = BabyBear;

// This creates a cubic extension field over Val using a binomial basis. It's used for generating challenges in the proof system.
// The reason why we want to extend our field for Challenges, is because the original Field size is too small to be brute-forced to solve the challenge.
type Challenge = BinomialExtensionField<Val, 4>;

// Your choice of Hash Function
type Perm = Poseidon2BabyBear<16>;
type MyHash = PaddingFreeSponge<Perm, 16, 8, 8>;

// Defines a compression function type using ByteHash, with 2 input blocks and 32-byte output.
type MyCompress = TruncatedPermutation<Perm, 2, 8, 16>;

// Defines an extension of the Merkle tree commitment scheme for the challenge field.
type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;

// Defines a Merkle tree commitment scheme for field elements with 32 levels.
type ValMmcs =
    MerkleTreeMmcs<<Val as Field>::Packing, <Val as Field>::Packing, MyHash, MyCompress, 8>;

// Defines the challenger type for generating random challenges.
type Challenger = DuplexChallenger<Val, Perm, 16, 8>;

// Defines the polynomial commitment scheme type.
type Dft = Radix2DitParallel<Val>;
type Pcs = TwoAdicFriPcs<Val, Dft, ValMmcs, ChallengeMmcs>;

// Defines the overall STARK configuration type.
type MyConfig = StarkConfig<Pcs, Challenge, Challenger>;

pub fn run_fibonacci_proof_bb() {
    let perm = Perm::new_from_rng_128(&mut thread_rng());
    let hash = MyHash::new(perm.clone());
    let compress = MyCompress::new(perm.clone());
    let val_mmcs = ValMmcs::new(hash, compress);

    // Creates an instance of the challenge Merkle tree commitment scheme.
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());

    // Configures the FRI (Fast Reed-Solomon IOP) protocol parameters.
    let fri_config = FriConfig {
        log_blowup: 1,
        num_queries: 100,
        proof_of_work_bits: 16,
        mmcs: challenge_mmcs,
    };

    // Instantiates the polynomial commitment scheme with the above parameters.
    let dft = Dft::default();
    let pcs = Pcs::new(dft, val_mmcs, fri_config);

    // Creates the STARK configuration instance.
    let config = MyConfig::new(pcs);

    // First define your AIR constraints inputs
    let num_steps = 8; // Choose the number of Fibonacci steps.
    let final_value = 21; // Choose the final Fibonacci value

    // Instantiate the AIR Scripts instance.
    let air = FibonacciAir {
        num_steps,
        final_value,
    };
    // Generate the execution trace, based on the inputs defined above.
    let trace = generate_fibonacci_trace::<Val>(num_steps);

    // Create Challenge sequence, in this case, we are using empty vector as seed inputs.
    let mut challenger = Challenger::new(perm.clone());

    // Generate your Proof!
    let proof = prove(&config, &air, &mut challenger, trace, &vec![]);

    // Create the same Challenge sequence as above for verification purpose
    let mut challenger = Challenger::new(perm);
    // Verify your proof!
    verify(&config, &air, &mut challenger, &proof, &vec![]).unwrap();

    println!("BabyBear Verified!")
}
