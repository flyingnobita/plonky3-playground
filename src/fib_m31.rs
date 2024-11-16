use std::marker::PhantomData;

use p3_challenger::{HashChallenger, SerializingChallenger32};
use p3_circle::CirclePcs;
use p3_commit::ExtensionMmcs;
use p3_field::extension::BinomialExtensionField;
use p3_fri::FriConfig;
use p3_keccak::Keccak256Hash;
use p3_merkle_tree::MerkleTreeMmcs;
use p3_mersenne_31::Mersenne31;
use p3_symmetric::{CompressionFunctionFromHasher, SerializingHasher32};
use p3_uni_stark::{prove, verify, StarkConfig};

use crate::{generate_fibonacci_trace, FibonacciAir};

// Your choice of Field
type Val = Mersenne31;

// This creates a cubic extension field over Val using a binomial basis. It's used for generating challenges in the proof system.
// The reason why we want to extend our field for Challenges, is because the original Field size is too small to be brute-forced to solve the challenge.
type Challenge = BinomialExtensionField<Val, 3>;

// Your choice of Hash Function
type ByteHash = Keccak256Hash;
// A serializer for Hash function, so that it can take Fields as inputs
type FieldHash = SerializingHasher32<ByteHash>;

// Defines a compression function type using ByteHash, with 2 input blocks and 32-byte output.
type MyCompress = CompressionFunctionFromHasher<ByteHash, 2, 32>;

// Defines an extension of the Merkle tree commitment scheme for the challenge field.
type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;

// Defines a Merkle tree commitment scheme for field elements with 32 levels.
type ValMmcs = MerkleTreeMmcs<Val, u8, FieldHash, MyCompress, 32>;

// Defines the challenger type for generating random challenges.
type Challenger = SerializingChallenger32<Val, HashChallenger<u8, ByteHash, 32>>;

// Defines the polynomial commitment scheme type.
type Pcs = CirclePcs<Val, ValMmcs, ChallengeMmcs>;

// Defines the overall STARK configuration type.
type MyConfig = StarkConfig<Pcs, Challenge, Challenger>;

pub fn run_fibonacci_proof_m31() {
    // Declaring an empty hash and its serializer.
    let byte_hash = ByteHash {};
    // Declaring Field hash function, it is used to hash field elements in the proof system
    let field_hash = FieldHash::new(Keccak256Hash {});

    // Creates a new instance of the compression function.
    let compress = MyCompress::new(byte_hash);

    // Instantiates the Merkle tree commitment scheme.
    let val_mmcs = ValMmcs::new(field_hash, compress);

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
    let pcs = Pcs {
        mmcs: val_mmcs,
        fri_config,
        _phantom: PhantomData,
    };

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
    let mut challenger = Challenger::from_hasher(vec![], byte_hash);

    // Generate your Proof!
    let proof = prove(&config, &air, &mut challenger, trace, &vec![]);

    // Create the same Challenge sequence as above for verification purpose
    let mut challenger = Challenger::from_hasher(vec![], byte_hash);
    // Verify your proof!
    verify(&config, &air, &mut challenger, &proof, &vec![]).unwrap();

    println!("M31 Verified!")
}
