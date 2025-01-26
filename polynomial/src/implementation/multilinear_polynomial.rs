// f(a,b) = 2ab + 3c
// create an array using the boolean hypercube. if the number of the unknowns is 3 it will be 2^3 = 8 which is equavalence to 1 << 3
// 000 001 010 011 100 101 110 111
// The a value will be the first bit, the b value will be the second bit, the c value will be the third bit

// using the boolean hypercube we can create a truth table for the function f(a,b,c) = 2ab + 3c
// 000 0
// 001 0
// 010 0
// 011 3
// 100 0
// 101 2
// 110 3
// 111 5

// using the truth table we can create a function that will return the value of the function f(a,b,c) = 2ab + 3c

// store the values in an array
// 0 0 0 3 0 2 3 5
// then partialy evaluate at a where b and c is constant in the boolean hypercube. 

fn evaluate_function(variables: &[u32]) -> u32 {
    // Example: f(a, b, c) = 2 * a * b + 3 * c
    2 * variables[0] * variables[1] + 3 * variables[2]
}

fn create_truth_table(num_variables: usize) -> Vec<u32> {
    let num_combinations = 1 << num_variables; // 2^num_variables combinations
    let mut truth_table = vec![];

    for combination in 0..num_combinations {
        let mut variables = vec![0; num_variables];
        for i in 0..num_variables {
            // Extract the value of each variable (bit by bit)
            
            variables[i] = (combination >> (num_variables - i - 1)) & 1;
        }
        truth_table.push(evaluate_function(&variables));
    }

    truth_table
}

fn partial_evaluation(
    truth_table: &[u32], // the original truth table value
    num_variables: usize, // 3 for 3 variables and 2 for 2 variables
    fixed_var_index: usize, //0,1,2 for 3 variables and  0,1 for 2 variables
    fixed_var_value: u32, // the value of a,b,c. a=4, b=1, c=2
) -> Vec<u32> {
    let subset_size = 1 << (num_variables - 1);
    let mut result = vec![0; subset_size];
    let bit_pos = num_variables - fixed_var_index - 1; // Correct bit position

    for i in 0..subset_size {
        let mask = (1 << bit_pos) - 1;
        let original_index_0 = (i & mask) | ((i >> bit_pos) << (bit_pos + 1));
        let original_index_1 = original_index_0 | (1 << bit_pos);

        let y1 = truth_table[original_index_0];
        let y2 = truth_table[original_index_1];
        result[i] = y1 + fixed_var_value * (y2 - y1);
    }
    result
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_create_truth_table(){
        let truth_table = create_truth_table(3); // 3 variables
        assert_eq!(truth_table.len(), 8); // 2^3 = 8
        assert_eq!(truth_table, vec![0, 3, 0, 3, 0, 3, 2, 5]);
    }

    #[test]
    fn test_partial_evaluation(){
        let truth_table = create_truth_table(3); // 3 variables
        let result = partial_evaluation(&truth_table, 3,0, 1); // Fix variable 1 to 1
        assert_eq!(result, vec![0, 3, 2, 5]);
    }
}




