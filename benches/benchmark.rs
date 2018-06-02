use std::sync::Arc;

#[macro_use]
extern crate criterion;
extern crate rand;
extern crate rayon;
extern crate wyrm;

use rayon::prelude::*;

use criterion::Criterion;

use wyrm::nn::lstm::*;
use wyrm::nn::xavier_normal;
use wyrm::*;

fn bench_node_reuse(c: &mut Criterion) {
    c.bench_function("node_reuse", |b| {
        let dim = 128;

        let x = ParameterNode::new(xavier_normal(1, dim));
        let y = ParameterNode::new(xavier_normal(dim, 10));
        let v = x.dot(&y);
        let z = v.clone() + v.clone() + v.clone() + v.clone();

        b.iter(|| {
            z.forward();
            z.zero_gradient();
        })
    });
}

fn bench_matrix_multiply(c: &mut Criterion) {
    c.bench_function("bench_matrix_multiply", |b| {
        let dim = 64;
        let num_epochs = 20;

        let x_data = Arc::new(HogwildParameter::new(xavier_normal(1, dim)));
        let y_data = Arc::new(HogwildParameter::new(xavier_normal(dim, 10)));

        b.iter(|| {
            (0..rayon::current_num_threads())
                .into_par_iter()
                .for_each(|_| {
                    let x = ParameterNode::shared(x_data.clone());
                    let y = ParameterNode::shared(y_data.clone());

                    let v = x.dot(&y);

                    for _ in 0..num_epochs {
                        v.forward();
                        v.zero_gradient();
                    }
                });
        })
    });
}

// fn bench_sofmax_exp_sum(b: &mut Criterion) {
//     c.bench_function("bench_softmax_exp_sum", |b| {
//         let x = vec![1.0; 32];
//         let max = 1.0;

//         b.iter(|| x.iter().map(|&x| wyrm::exp(x - max)).sum::<f32>().ln())
//     })
// }

// #[bench]
// fn bench_sofmax_exp_sum_unrolled(b: &mut Criterion) {
//     let x = vec![1.0; 32];
//     let max = 1.0;

//     b.iter(|| softmax_exp_sum(&x, max).ln())
// }

// fn bench_exp(b: &mut Criterion) {
//     let x: Vec<f32> = vec![1.0; 32];

//     let mut v = 0.0;

//     b.iter(|| x.iter().for_each(|&y| v += y.exp()));
// }

// fn bench_fastexp(b: &mut Criterion) {
//     let x: Vec<f32> = vec![1.0; 32];

//     let mut v = 0.0;

//     b.iter(|| x.iter().for_each(|&y| v += fastexp(y)));
// }

// fn bench_dot(b: &mut Criterion) {
//     let xs = vec![0.0; 256];
//     let ys = vec![0.0; 256];

//     b.iter(|| dot(&xs[..], &ys[..]));
// }

// fn bench_unrolled_dot(b: &mut Criterion) {
//     let xs = vec![0.0; 256];
//     let ys = vec![0.0; 256];

//     b.iter(|| unrolled_dot(&xs[..], &ys[..]));
// }

// fn bench_simd_dot(b: &mut Criterion) {
//     let xs = vec![0.0; 256];
//     let ys = vec![0.0; 256];

//     b.iter(|| simd_dot(&xs[..], &ys[..]));
// }

// fn bench_array_scaled_assign(b: &mut Criterion) {
//     let mut xs = random_matrix(256, 1);
//     let ys = random_matrix(256, 1);

//     b.iter(|| array_scaled_assign(&mut xs, &ys, 3.5));
// }

// fn bench_slice_scaled_assign(b: &mut Criterion) {
//     let mut xs = random_matrix(256, 1);
//     let ys = random_matrix(256, 1);

//     b.iter(|| scaled_assign(&mut xs, &ys, 3.5));
// }

// fn bench_array_assign(b: &mut Criterion) {
//     let mut xs = random_matrix(256, 1);
//     let ys = random_matrix(256, 1);

//     b.iter(|| array_assign(&mut xs, &ys));
// }

// fn bench_slice_assign(b: &mut Criterion) {
//     let mut xs = random_matrix(256, 1);
//     let ys = random_matrix(256, 1);

//     b.iter(|| assign(&mut xs, &ys));
// }

// fn dot_node_specializations_mm(b: &mut Criterion) {
//     let x = random_matrix(64, 64);
//     let y = random_matrix(64, 64);
//     let mut z = random_matrix(64, 64);

//     b.iter(|| mat_mul(1.0, &x, &y, 0.0, &mut z));
// }

// fn dot_node_general_vm(b: &mut Criterion) {
//     let x = random_matrix(1, 64);
//     let y = random_matrix(64, 64);
//     let mut z = random_matrix(1, 64);

//     b.iter(|| general_mat_mul(1.0, &x, &y, 0.0, &mut z));
// }

// fn dot_node_specializations_vm(b: &mut Criterion) {
//     let x = random_matrix(1, 64);
//     let y = random_matrix(64, 64);
//     let mut z = random_matrix(1, 64);

//     b.iter(|| mat_mul(1.0, &x, &y, 0.0, &mut z));
// }

// fn dot_node_specializations_mv(b: &mut Criterion) {
//     let x = random_matrix(64, 64);
//     let y = random_matrix(64, 1);
//     let mut z = random_matrix(64, 1);

//     b.iter(|| mat_mul(1.0, &x, &y, 0.0, &mut z));
// }

// fn dot_node_general_mv(b: &mut Criterion) {
//     let x = random_matrix(64, 64);
//     let y = random_matrix(64, 1);
//     let mut z = random_matrix(64, 1);

//     b.iter(|| general_mat_mul(1.0, &x, &y, 0.0, &mut z));
// }

fn pi_digits(num: usize) -> Vec<usize> {
    let pi_str = include_str!("../src/nn/pi.txt");
    pi_str
        .chars()
        .filter_map(|x| x.to_digit(10))
        .map(|x| x as usize)
        .take(num)
        .collect()
}

fn bench_lstm_simple_shallow(c: &mut Criterion) {
    c.bench_function("bench_lstm_simple_shallow", |b| {
        let input_dim = 10;
        let hidden_dim = 5;

        // Initialize the parameters.
        let lstm_params = Parameters::new(input_dim, hidden_dim, &mut rand::thread_rng());
        let lstm = lstm_params.build_cell();

        // Initialize the cell state and hidden state.
        let state = InputNode::new(Arr::zeros((1, hidden_dim)));
        let hidden = InputNode::new(Arr::zeros((1, hidden_dim)));

        // Construct the input node.
        let input = InputNode::new(xavier_normal(1, input_dim));

        // The forward method outputs a tuple of (cell_state, hidden_state).
        let mut state = lstm.forward((state, hidden), input.clone());

        // Construct a shallow RNN.
        for _ in 0..10 {
            state = lstm.forward(state.clone(), input.clone());
        }

        // Unpack the cell and hidden state.
        let (_, mut hidden) = state;

        b.iter(|| {
            for _ in 0..20 {
                hidden.forward();
                hidden.backward(1.0);
            }
        })
    });
}

fn bench_lstm_simple_deep(c: &mut Criterion) {
    c.bench_function("bench_lstm_simple_deep", |b| {
        let input_dim = 10;
        let hidden_dim = 5;

        // Initialize the parameters.
        let lstm_params = Parameters::new(input_dim, hidden_dim, &mut rand::thread_rng());
        let lstm = lstm_params.build_cell();

        // Initialize the cell state and hidden state.
        let state = InputNode::new(Arr::zeros((1, hidden_dim)));
        let hidden = InputNode::new(Arr::zeros((1, hidden_dim)));

        // Construct the input node.
        let input = InputNode::new(xavier_normal(1, input_dim));

        // The forward method outputs a tuple of (cell_state, hidden_state).
        let mut state = lstm.forward((state, hidden), input.clone());

        // Construct a shallow RNN.
        for _ in 0..200 {
            state = lstm.forward(state.clone(), input.clone());
        }

        // Unpack the cell and hidden state.
        let (_, mut hidden) = state;

        b.iter(|| {
            hidden.forward();
            hidden.backward(1.0);
        })
    });
}

criterion_group!(
    benches,
    bench_node_reuse,
    bench_matrix_multiply,
    bench_lstm_simple_shallow,
    bench_lstm_simple_deep
);
criterion_main!(benches);
