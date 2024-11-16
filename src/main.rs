use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;

mod fib_bb;
mod fib_m31;

fn main() {
    fib_m31::run_fibonacci_proof_m31();
    fib_bb::run_fibonacci_proof_bb();
}

// Define your AIR constraints inputs via declaring a Struct with relevant inputs inside it.
pub struct FibonacciAir {
    pub num_steps: usize, // numbers of steps to run the fibonacci iterations
    pub final_value: u32, // precomputed final result after the numbers of steps.
}

// Define your Execution Trace Row size,
impl<F: Field> BaseAir<F> for FibonacciAir {
    fn width(&self) -> usize {
        2 // Row width for fibonacci program is 2
    }
}

// Define your Constraints
impl<AB: AirBuilder> Air<AB> for FibonacciAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0); // get the current row
        let next = main.row_slice(1); // get the next row

        // Enforce starting values
        builder.when_first_row().assert_eq(local[0], AB::Expr::ZERO);
        builder.when_first_row().assert_eq(local[1], AB::Expr::ONE);

        // Enforce state transition constraints
        builder.when_transition().assert_eq(next[0], local[1]);
        builder
            .when_transition()
            .assert_eq(next[1], local[0] + local[1]);

        // Constrain the final value
        let final_value = AB::Expr::from_canonical_u32(self.final_value);
        builder.when_last_row().assert_eq(local[1], final_value);
    }
}

pub fn generate_fibonacci_trace<F: Field>(num_steps: usize) -> RowMajorMatrix<F> {
    // Declaring the total fields needed to keep track of the execution with the given parameter, which in this case, is num_steps multiply by 2, where 2 is the width of the AIR scripts.
    let mut values = Vec::with_capacity(num_steps * 2);

    // Define your initial state, 0 and 1.
    let mut a = F::ZERO;
    let mut b = F::ONE;

    // Run your program and fill in the states in each iteration in the `values` vector
    for _ in 0..num_steps {
        values.push(a);
        values.push(b);
        let c = a + b;
        a = b;
        b = c;
    }

    // Convert it into 2D matrix.
    RowMajorMatrix::new(values, 2)
}
