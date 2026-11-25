    use super::{AngleMode, CalcError, Calculator, Complex, Matrix, Value};

    fn matrix(rows: usize, cols: usize, data: &[f64]) -> Matrix {
        let complex_data = data
            .iter()
            .map(|value| Complex {
                re: *value,
                im: 0.0,
            })
            .collect::<Vec<_>>();
        Matrix::new(rows, cols, complex_data).expect("valid matrix")
    }

    fn assert_real_close(actual: f64, expected: f64, eps: f64) {
        assert!(
            (actual - expected).abs() <= eps,
            "expected {expected}, got {actual}"
        );
    }

    fn assert_matrix_close(actual: &Matrix, expected: &Matrix, eps: f64) {
        assert_eq!(actual.rows, expected.rows);
        assert_eq!(actual.cols, expected.cols);
        for (a, e) in actual.data.iter().zip(&expected.data) {
            assert_real_close(a.re, e.re, eps);
            assert_real_close(a.im, e.im, eps);
        }
    }

    #[test]
    fn enter_pushes_real_and_clears_entry() {
        let mut calc = Calculator::new();
        calc.entry_set("12.5");

        let result = calc.enter();

        assert_eq!(result, Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(12.5)]);
        assert_eq!(calc.state().entry_buffer, "");
    }

    #[test]
    fn enter_with_invalid_input_preserves_state() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(9.0));
        calc.entry_set("abc");

        let result = calc.enter();

        assert_eq!(
            result,
            Err(CalcError::InvalidInput(
                "entry buffer is not a valid number".to_string()
            ))
        );
        assert_eq!(calc.state().stack, vec![Value::Real(9.0)]);
        assert_eq!(calc.state().entry_buffer, "abc");
    }

    #[test]
    fn drop_returns_top_value() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(3.0));
        calc.push_value(Value::Real(7.0));

        let dropped = calc.drop();

        assert_eq!(dropped, Ok(Value::Real(7.0)));
        assert_eq!(calc.state().stack, vec![Value::Real(3.0)]);
    }

    #[test]
    fn dup_copies_top_value() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(2.0));

        let result = calc.dup();

        assert_eq!(result, Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(2.0), Value::Real(2.0)]);
    }

    #[test]
    fn swap_exchanges_top_two_values() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(1.0));
        calc.push_value(Value::Real(2.0));

        let result = calc.swap();

        assert_eq!(result, Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(2.0), Value::Real(1.0)]);
    }

    #[test]
    fn rot_rotates_top_three_values_left() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(1.0));
        calc.push_value(Value::Real(2.0));
        calc.push_value(Value::Real(3.0));

        let result = calc.rot();

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Real(2.0), Value::Real(3.0), Value::Real(1.0)]
        );
    }

    #[test]
    fn stack_underflow_errors_do_not_modify_stack() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(10.0));

        let dup_before = calc.state().stack.clone();
        let swap_result = calc.swap();

        assert_eq!(
            swap_result,
            Err(CalcError::StackUnderflow {
                needed: 2,
                available: 1
            })
        );
        assert_eq!(calc.state().stack, dup_before);
    }

    #[test]
    fn add_real_values() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(10.0));
        calc.push_value(Value::Real(5.0));

        let result = calc.add();

        assert_eq!(result, Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(15.0)]);
    }

    #[test]
    fn add_mixed_values_promotes_to_complex() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(2.0));
        calc.push_value(Value::Complex(Complex { re: 3.0, im: 4.0 }));

        let result = calc.add();

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Complex(Complex { re: 5.0, im: 4.0 })]
        );
    }

    #[test]
    fn div_by_zero_preserves_stack() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(12.0));
        calc.push_value(Value::Real(0.0));
        let before = calc.state().stack.clone();

        let result = calc.div();

        assert_eq!(result, Err(CalcError::DivideByZero));
        assert_eq!(calc.state().stack, before);
    }

    #[test]
    fn sqrt_negative_real_preserves_stack() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(-9.0));
        let before = calc.state().stack.clone();

        let result = calc.sqrt();

        assert_eq!(
            result,
            Err(CalcError::DomainError(
                "sqrt is undefined for negative real values".to_string()
            ))
        );
        assert_eq!(calc.state().stack, before);
    }

    #[test]
    fn sin_respects_degree_mode_for_real_values() {
        let mut calc = Calculator::new();
        calc.set_angle_mode(AngleMode::Deg);
        calc.push_value(Value::Real(90.0));

        let result = calc.sin();

        assert_eq!(result, Ok(()));
        match calc.state().stack.last() {
            Some(Value::Real(v)) => assert!((v - 1.0).abs() < 1e-12),
            other => panic!("unexpected stack value: {other:?}"),
        }
    }

    #[test]
    fn ln_non_positive_real_preserves_stack() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(0.0));
        let before = calc.state().stack.clone();

        let result = calc.ln();

        assert_eq!(
            result,
            Err(CalcError::DomainError(
                "ln is undefined for non-positive real values".to_string()
            ))
        );
        assert_eq!(calc.state().stack, before);
    }

    #[test]
    fn add_two_matrices() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, 2.0, 3.0, 4.0])));
        calc.push_value(Value::Matrix(matrix(2, 2, &[5.0, 6.0, 7.0, 8.0])));

        let result = calc.add();

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(2, 2, &[6.0, 8.0, 10.0, 12.0]))]
        );
    }

    #[test]
    fn hadamard_mul_and_div() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(1, 3, &[1.0, 2.0, 3.0])));
        calc.push_value(Value::Matrix(matrix(1, 3, &[4.0, 5.0, 6.0])));

        assert_eq!(calc.hadamard_mul(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(1, 3, &[4.0, 10.0, 18.0]))]
        );

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 3, &[8.0, 10.0, 18.0])));
        calc.push_value(Value::Matrix(matrix(1, 3, &[2.0, 5.0, 3.0])));

        assert_eq!(calc.hadamard_div(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(1, 3, &[4.0, 2.0, 6.0]))]
        );

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 3, &[1.0, -2.0, 3.0])));
        calc.push_value(Value::Real(2.0));

        assert_eq!(calc.hadamard_mul(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(1, 3, &[2.0, -4.0, 6.0]))]
        );

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 3, &[2.0, 4.0, 8.0])));
        calc.push_value(Value::Real(2.0));

        assert_eq!(calc.hadamard_div(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(1, 3, &[1.0, 2.0, 4.0]))]
        );
    }

    #[test]
    fn matrix_add_shape_mismatch_preserves_stack() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, 2.0, 3.0, 4.0])));
        calc.push_value(Value::Matrix(matrix(1, 3, &[5.0, 6.0, 7.0])));
        let before = calc.state().stack.clone();

        let result = calc.add();

        assert!(
            matches!(result, Err(CalcError::TypeMismatch(message)) if message.contains("equal matrix dimensions"))
        );
        assert_eq!(calc.state().stack, before);
    }

    #[test]
    fn mul_two_matrices() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0])));
        calc.push_value(Value::Matrix(matrix(
            3,
            2,
            &[7.0, 8.0, 9.0, 10.0, 11.0, 12.0],
        )));

        let result = calc.mul();

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(2, 2, &[58.0, 64.0, 139.0, 154.0]))]
        );
    }

    #[test]
    fn matrix_times_scalar() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, -2.0, 3.0, -4.0])));
        calc.push_value(Value::Real(2.5));

        let result = calc.mul();

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(2, 2, &[2.5, -5.0, 7.5, -10.0]))]
        );
    }

    #[test]
    fn transpose_matrix() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0])));

        let result = calc.transpose();

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(3, 2, &[1.0, 4.0, 2.0, 5.0, 3.0, 6.0]))]
        );
    }

    #[test]
    fn push_identity_matrix() {
        let mut calc = Calculator::new();

        let result = calc.push_identity(3);

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(
                3,
                3,
                &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
            ))]
        );
    }

    #[test]
    fn stack_vec_converts_scalars_to_column_vector() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(1.0));
        calc.push_value(Value::Complex(Complex { re: 2.0, im: -1.0 }));
        calc.push_value(Value::Real(3.5));

        assert_eq!(calc.stack_vec(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(
                Matrix::new(
                    3,
                    1,
                    vec![
                        Complex { re: 1.0, im: 0.0 },
                        Complex { re: 2.0, im: -1.0 },
                        Complex { re: 3.5, im: 0.0 },
                    ],
                )
                .expect("valid matrix")
            )]
        );
    }

    #[test]
    fn stack_vec_rejects_matrix_values_and_preserves_stack() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(1.0));
        calc.push_value(Value::Matrix(matrix(1, 1, &[2.0])));
        let before = calc.state().stack.clone();

        let result = calc.stack_vec();

        assert!(matches!(result, Err(CalcError::TypeMismatch(_))));
        assert_eq!(calc.state().stack, before);
    }

    #[test]
    fn ravel_matrix_to_vector_and_vector_to_scalar_stack() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, 2.0, 3.0, 4.0])));

        assert_eq!(calc.ravel(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(4, 1, &[1.0, 2.0, 3.0, 4.0]))]
        );

        calc.clear_all();
        let complex_vector = Matrix::new(
            1,
            3,
            vec![
                Complex { re: 1.0, im: 0.0 },
                Complex { re: 2.0, im: -1.0 },
                Complex { re: 3.0, im: 0.0 },
            ],
        )
        .expect("valid matrix");
        calc.push_value(Value::Matrix(complex_vector));

        assert_eq!(calc.ravel(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![
                Value::Real(1.0),
                Value::Complex(Complex { re: 2.0, im: -1.0 }),
                Value::Real(3.0),
            ]
        );
    }

    #[test]
    fn hstack_and_vstack_combine_scalars_and_matrices() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(1.0));
        calc.push_value(Value::Real(2.0));
        calc.push_value(Value::Real(3.0));
        calc.push_value(Value::Real(3.0));
        assert_eq!(calc.hstack(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(1, 3, &[1.0, 2.0, 3.0]))]
        );

        calc.clear_all();
        calc.push_value(Value::Real(1.0));
        calc.push_value(Value::Real(2.0));
        calc.push_value(Value::Real(3.0));
        calc.push_value(Value::Real(3.0));
        assert_eq!(calc.vstack(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(3, 1, &[1.0, 2.0, 3.0]))]
        );

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, 2.0, 3.0, 4.0])));
        calc.push_value(Value::Matrix(matrix(2, 2, &[5.0, 6.0, 7.0, 8.0])));
        calc.push_value(Value::Real(2.0));
        assert_eq!(calc.hstack(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(
                2,
                4,
                &[1.0, 2.0, 5.0, 6.0, 3.0, 4.0, 7.0, 8.0]
            ))]
        );

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, 2.0, 3.0, 4.0])));
        calc.push_value(Value::Matrix(matrix(2, 2, &[5.0, 6.0, 7.0, 8.0])));
        calc.push_value(Value::Real(2.0));
        assert_eq!(calc.vstack(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(
                4,
                2,
                &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]
            ))]
        );
    }

    #[test]
    fn hravel_and_vravel_split_matrices_and_vectors() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0])));
        assert_eq!(calc.hravel(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![
                Value::Matrix(matrix(2, 1, &[1.0, 4.0])),
                Value::Matrix(matrix(2, 1, &[2.0, 5.0])),
                Value::Matrix(matrix(2, 1, &[3.0, 6.0])),
            ]
        );

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0])));
        assert_eq!(calc.vravel(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![
                Value::Matrix(matrix(1, 3, &[1.0, 2.0, 3.0])),
                Value::Matrix(matrix(1, 3, &[4.0, 5.0, 6.0])),
            ]
        );

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 3, &[7.0, 8.0, 9.0])));
        assert_eq!(calc.hravel(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Real(7.0), Value::Real(8.0), Value::Real(9.0)]
        );
    }

    #[test]
    fn matrix_and_complex_multiplication_scales_matrix() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(1, 1, &[3.0])));
        calc.push_value(Value::Complex(Complex { re: 2.0, im: 1.0 }));

        let result = calc.mul();

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(
                Matrix::new(1, 1, vec![Complex { re: 6.0, im: 3.0 }]).expect("valid matrix")
            )]
        );
    }

    #[test]
    fn matrix_scalar_add_sub_div_and_pow() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(1, 2, &[2.0, 4.0])));
        calc.push_value(Value::Real(3.0));
        assert_eq!(calc.add(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(1, 2, &[5.0, 7.0]))]
        );

        calc.clear_all();
        calc.push_value(Value::Real(10.0));
        calc.push_value(Value::Matrix(matrix(1, 2, &[2.0, 3.0])));
        assert_eq!(calc.sub(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(1, 2, &[8.0, 7.0]))]
        );

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 2, &[6.0, 8.0])));
        calc.push_value(Value::Real(2.0));
        assert_eq!(calc.div(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(1, 2, &[3.0, 4.0]))]
        );

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 2, &[2.0, 3.0])));
        calc.push_value(Value::Real(2.0));
        assert_eq!(calc.pow(), Ok(()));
        let expected = matrix(1, 2, &[4.0, 9.0]);
        match calc.state().stack.as_slice() {
            [Value::Matrix(actual)] => assert_matrix_close(actual, &expected, 1e-12),
            other => panic!("expected matrix on stack, got {other:?}"),
        }
    }

    #[test]
    fn conjugate_supports_matrix_values() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(
            Matrix::new(
                1,
                2,
                vec![Complex { re: 1.0, im: 2.0 }, Complex { re: -3.0, im: -4.5 }],
            )
            .expect("valid matrix"),
        ));

        assert_eq!(calc.conjugate(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(
                Matrix::new(
                    1,
                    2,
                    vec![Complex { re: 1.0, im: -2.0 }, Complex { re: -3.0, im: 4.5 },],
                )
                .expect("valid matrix")
            )]
        );
    }

    #[test]
    fn dot_cross_trace_and_norm_p() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(3, 1, &[1.0, 2.0, 3.0])));
        calc.push_value(Value::Matrix(matrix(1, 3, &[4.0, 5.0, 6.0])));

        assert_eq!(calc.dot(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Complex(Complex { re: 32.0, im: 0.0 })]
        );

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 3, &[1.0, 0.0, 0.0])));
        calc.push_value(Value::Matrix(matrix(1, 3, &[0.0, 1.0, 0.0])));

        assert_eq!(calc.cross(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(1, 3, &[0.0, 0.0, 1.0]))]
        );

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, 2.0, 3.0, 4.0])));

        assert_eq!(calc.trace(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Complex(Complex { re: 5.0, im: 0.0 })]
        );

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 2, &[3.0, 4.0])));
        calc.push_value(Value::Real(2.0));

        assert_eq!(calc.norm_p(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Real(v)] => assert_real_close(*v, 5.0, 1e-12),
            other => panic!("expected real norm value, got {other:?}"),
        }
    }

    #[test]
    fn vector_statistics_ops() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(1, 5, &[1.0, 2.0, 2.0, 4.0, 5.0])));

        assert_eq!(calc.mean(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Real(v)] => assert_real_close(*v, 2.8, 1e-12),
            other => panic!("expected real mean value, got {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 5, &[1.0, 2.0, 2.0, 4.0, 5.0])));
        assert_eq!(calc.mode(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Real(v)] => assert_real_close(*v, 2.0, 1e-12),
            other => panic!("expected real mode value, got {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 2, &[3.0, 4.0])));
        assert_eq!(calc.variance(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Real(v)] => assert_real_close(*v, 0.25, 1e-12),
            other => panic!("expected real variance value, got {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 2, &[3.0, 4.0])));
        assert_eq!(calc.std_dev_p(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Real(v)] => assert_real_close(*v, 0.5, 1e-12),
            other => panic!("expected real std_dev_p value, got {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 5, &[1.0, 2.0, 2.0, 4.0, 5.0])));
        assert_eq!(calc.max_value(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Real(v)] => assert_real_close(*v, 5.0, 1e-12),
            other => panic!("expected real max value, got {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 5, &[1.0, 2.0, 2.0, 4.0, 5.0])));
        assert_eq!(calc.min_value(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Real(v)] => assert_real_close(*v, 1.0, 1e-12),
            other => panic!("expected real min value, got {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Real(1.0));
        calc.push_value(Value::Real(2.0));
        calc.push_value(Value::Real(5.0));
        assert_eq!(calc.mean(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Real(v)] => assert_real_close(*v, 8.0 / 3.0, 1e-12),
            other => panic!("expected scalar-stack mean value, got {other:?}"),
        }
    }

    #[test]
    fn scalar_complex_rounding_ops_apply_elementwise_to_matrices() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(1, 3, &[3.0, 4.0, 5.0])));
        calc.push_value(Value::Real(2.0));
        assert_eq!(calc.pow(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Matrix(actual)] => {
                let expected = matrix(1, 3, &[9.0, 16.0, 25.0]);
                assert_matrix_close(actual, &expected, 1e-12);
            }
            other => panic!("expected matrix after elementwise pow, got {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 3, &[-3.0, 0.0, 4.0])));
        assert_eq!(calc.abs(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Matrix(actual)] => {
                let expected = matrix(1, 3, &[3.0, 0.0, 4.0]);
                assert_matrix_close(actual, &expected, 1e-12);
            }
            other => panic!("expected matrix after elementwise abs, got {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 3, &[180.0, 90.0, 0.0])));
        assert_eq!(calc.to_rad(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Matrix(actual)] => {
                let expected = matrix(
                    1,
                    3,
                    &[std::f64::consts::PI, std::f64::consts::FRAC_PI_2, 0.0],
                );
                assert_matrix_close(actual, &expected, 1e-12);
            }
            other => panic!("expected matrix after elementwise to_rad, got {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 3, &[1.2, -2.5, 3.8])));
        assert_eq!(calc.round_value(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Matrix(actual)] => {
                let expected = matrix(1, 3, &[1.0, -3.0, 4.0]);
                assert_matrix_close(actual, &expected, 1e-12);
            }
            other => panic!("expected matrix after elementwise round, got {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 2, &[50.0, 10.0])));
        calc.push_value(Value::Real(20.0));
        assert_eq!(calc.percent(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Matrix(actual)] => {
                let expected = matrix(1, 2, &[10.0, 2.0]);
                assert_matrix_close(actual, &expected, 1e-12);
            }
            other => panic!("expected matrix after elementwise percent, got {other:?}"),
        }
    }

    #[test]
    fn diag_and_mat_exp() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(1, 3, &[1.0, 2.0, 3.0])));

        assert_eq!(calc.diag(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Matrix(actual)] => {
                let expected = matrix(3, 3, &[1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 3.0]);
                assert_matrix_close(actual, &expected, 1e-12);
            }
            other => panic!("expected matrix diag value, got {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(1, 3, &[1.0, 2.0, 3.0])));

        assert_eq!(calc.toep(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Matrix(actual)] => {
                let expected = matrix(3, 3, &[1.0, 2.0, 3.0, 2.0, 1.0, 2.0, 3.0, 2.0, 1.0]);
                assert_matrix_close(actual, &expected, 1e-12);
            }
            other => panic!("expected matrix toep value, got {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, 0.0, 0.0, 2.0])));

        assert_eq!(calc.mat_exp(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Matrix(actual)] => {
                let expected = matrix(
                    2,
                    2,
                    &[std::f64::consts::E, 0.0, 0.0, std::f64::consts::E.powi(2)],
                );
                assert_matrix_close(actual, &expected, 1e-10);
            }
            other => panic!("expected matrix MatExp value, got {other:?}"),
        }
    }

    #[test]
    fn hermitian_and_mat_pow() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(
            Matrix::new(
                2,
                2,
                vec![
                    Complex { re: 1.0, im: 2.0 },
                    Complex { re: 3.0, im: -1.0 },
                    Complex { re: -4.0, im: 0.5 },
                    Complex { re: 2.0, im: 0.0 },
                ],
            )
            .expect("valid matrix"),
        ));

        assert_eq!(calc.hermitian(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Matrix(actual)] => {
                let expected = Matrix::new(
                    2,
                    2,
                    vec![
                        Complex { re: 1.0, im: -2.0 },
                        Complex { re: -4.0, im: -0.5 },
                        Complex { re: 3.0, im: 1.0 },
                        Complex { re: 2.0, im: -0.0 },
                    ],
                )
                .expect("valid matrix");
                assert_matrix_close(actual, &expected, 1e-12);
            }
            other => panic!("expected Hermitian matrix, got {other:?}"),
        }

        calc.clear_all();
        let base = matrix(2, 2, &[2.0, 0.0, 0.0, 3.0]);
        calc.push_value(Value::Matrix(base.clone()));
        calc.push_value(Value::Real(3.0));
        assert_eq!(calc.mat_pow(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Matrix(actual)] => {
                let expected = matrix(2, 2, &[8.0, 0.0, 0.0, 27.0]);
                assert_matrix_close(actual, &expected, 1e-12);
            }
            other => panic!("expected MatPow matrix, got {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Matrix(base));
        calc.push_value(Value::Real(-1.0));
        assert_eq!(calc.mat_pow(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Matrix(actual)] => {
                let expected = matrix(2, 2, &[0.5, 0.0, 0.0, 1.0 / 3.0]);
                assert_matrix_close(actual, &expected, 1e-12);
            }
            other => panic!("expected inverse MatPow matrix, got {other:?}"),
        }
    }

    #[test]
    fn qr_and_lu_decompose() {
        let mut calc = Calculator::new();
        let original_qr = matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        calc.push_value(Value::Matrix(original_qr.clone()));

        assert_eq!(calc.qr(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Matrix(q), Value::Matrix(r)] => {
                let reconstructed = Calculator::matrix_mul(q, r).expect("q*r");
                assert_matrix_close(&reconstructed, &original_qr, 1e-10);
            }
            other => panic!("expected Q and R on stack, got {other:?}"),
        }

        calc.clear_all();
        let original_lu = matrix(2, 2, &[4.0, 3.0, 6.0, 3.0]);
        calc.push_value(Value::Matrix(original_lu.clone()));
        assert_eq!(calc.lu(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Matrix(p), Value::Matrix(l), Value::Matrix(u)] => {
                let pa = Calculator::matrix_mul(p, &original_lu).expect("p*a");
                let lu = Calculator::matrix_mul(l, u).expect("l*u");
                assert_matrix_close(&pa, &lu, 1e-10);
            }
            other => panic!("expected P, L and U on stack, got {other:?}"),
        }

        calc.clear_all();
        let complex_lu = Matrix::new(
            2,
            2,
            vec![
                Complex { re: 1.0, im: 1.0 },
                Complex { re: 2.0, im: -0.5 },
                Complex { re: 0.5, im: 0.0 },
                Complex { re: 3.0, im: 2.0 },
            ],
        )
        .expect("valid matrix");
        calc.push_value(Value::Matrix(complex_lu.clone()));
        assert_eq!(calc.lu(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Matrix(p), Value::Matrix(l), Value::Matrix(u)] => {
                let pa = Calculator::matrix_mul(p, &complex_lu).expect("p*a");
                let lu = Calculator::matrix_mul(l, u).expect("l*u");
                assert_matrix_close(&pa, &lu, 1e-8);
            }
            other => panic!("expected P, L and U on stack, got {other:?}"),
        }
    }

    #[test]
    fn svd_decompose_reconstructs_matrix() {
        let mut calc = Calculator::new();
        let original = matrix(2, 2, &[3.0, 1.0, 1.0, 3.0]);
        calc.push_value(Value::Matrix(original.clone()));

        assert_eq!(calc.svd(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Matrix(u), Value::Matrix(s), Value::Matrix(vt)] => {
                let us = Calculator::matrix_mul(u, s).expect("u*s");
                let reconstructed = Calculator::matrix_mul(&us, vt).expect("(u*s)*vt");
                assert_matrix_close(&reconstructed, &original, 1e-8);
            }
            other => panic!("expected U, S and Vt on stack, got {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Matrix(
            Matrix::new(
                2,
                2,
                vec![
                    Complex { re: 1.0, im: 2.0 },
                    Complex { re: 0.0, im: -1.0 },
                    Complex { re: 3.0, im: 0.5 },
                    Complex { re: -2.0, im: 0.0 },
                ],
            )
            .expect("valid matrix"),
        ));

        assert_eq!(calc.svd(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Matrix(u), Value::Matrix(s), Value::Matrix(vt)] => {
                let us = Calculator::matrix_mul(u, s).expect("u*s");
                let reconstructed = Calculator::matrix_mul(&us, vt).expect("(u*s)*vt");
                let expected = Matrix::new(
                    2,
                    2,
                    vec![
                        Complex { re: 1.0, im: 2.0 },
                        Complex { re: 0.0, im: -1.0 },
                        Complex { re: 3.0, im: 0.5 },
                        Complex { re: -2.0, im: 0.0 },
                    ],
                )
                .expect("valid matrix");
                assert_matrix_close(&reconstructed, &expected, 1e-8);
            }
            other => panic!("expected U, S and Vt on stack, got {other:?}"),
        }
    }

    #[test]
    fn evd_decompose_and_warning_path() {
        let mut calc = Calculator::new();
        let diagonal = matrix(2, 2, &[2.0, 0.0, 0.0, 3.0]);
        calc.push_value(Value::Matrix(diagonal.clone()));

        let warning = calc.evd().expect("evd should succeed");
        assert!(warning.is_none());
        match calc.state().stack.as_slice() {
            [Value::Matrix(v), Value::Matrix(d)] => {
                let v_inv = Calculator::matrix_inverse(v).expect("invertible eigenvectors");
                let vd = Calculator::matrix_mul(v, d).expect("v*d");
                let reconstructed = Calculator::matrix_mul(&vd, &v_inv).expect("(v*d)*v^-1");
                assert_matrix_close(&reconstructed, &diagonal, 1e-8);
            }
            other => panic!("expected V and D on stack, got {other:?}"),
        }

        calc.clear_all();
        let complex_diagonal = Matrix::new(
            2,
            2,
            vec![
                Complex { re: 2.0, im: 1.0 },
                Complex { re: 0.0, im: 0.0 },
                Complex { re: 0.0, im: 0.0 },
                Complex { re: -1.0, im: 0.5 },
            ],
        )
        .expect("valid matrix");
        calc.push_value(Value::Matrix(complex_diagonal.clone()));
        let warning = calc.evd().expect("evd should succeed");
        assert!(warning.is_none());
        match calc.state().stack.as_slice() {
            [Value::Matrix(v), Value::Matrix(d)] => {
                let v_inv = Calculator::matrix_inverse(v).expect("invertible eigenvectors");
                let vd = Calculator::matrix_mul(v, d).expect("v*d");
                let reconstructed = Calculator::matrix_mul(&vd, &v_inv).expect("(v*d)*v^-1");
                assert_matrix_close(&reconstructed, &complex_diagonal, 1e-8);
            }
            other => panic!("expected V and D on stack, got {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, 1.0, 0.0, 1.0])));
        let warning = calc.evd().expect("evd should return fallback");
        assert!(warning.is_some());
        assert_eq!(calc.state().stack.len(), 2);
    }

    #[test]
    fn determinant_of_square_matrix() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, 2.0, 3.0, 4.0])));

        let result = calc.determinant();

        assert_eq!(result, Ok(()));
        match calc.state().stack.last() {
            Some(Value::Complex(v)) => {
                assert_real_close(v.re, -2.0, 1e-12);
                assert_real_close(v.im, 0.0, 1e-12);
            }
            other => panic!("unexpected stack value: {other:?}"),
        }
    }

    #[test]
    fn inverse_of_square_matrix() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[4.0, 7.0, 2.0, 6.0])));

        let result = calc.inverse();

        assert_eq!(result, Ok(()));
        match calc.state().stack.last() {
            Some(Value::Matrix(actual)) => {
                let expected = matrix(2, 2, &[0.6, -0.7, -0.2, 0.4]);
                assert_matrix_close(actual, &expected, 1e-12);
            }
            other => panic!("unexpected stack value: {other:?}"),
        }
    }

    #[test]
    fn solve_ax_b_with_vector_rhs() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[3.0, 2.0, 1.0, 2.0])));
        calc.push_value(Value::Matrix(matrix(2, 1, &[5.0, 5.0])));

        let result = calc.solve_ax_b();

        assert_eq!(result, Ok(()));
        match calc.state().stack.last() {
            Some(Value::Matrix(actual)) => {
                let expected = matrix(2, 1, &[0.0, 2.5]);
                assert_matrix_close(actual, &expected, 1e-12);
            }
            other => panic!("unexpected stack value: {other:?}"),
        }
    }

    #[test]
    fn solve_lstsq_with_overdetermined_rhs() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(3, 2, &[1.0, 0.0, 0.0, 1.0, 1.0, 1.0])));
        calc.push_value(Value::Matrix(matrix(3, 1, &[1.0, 2.0, 3.0])));

        let warning = calc.solve_lstsq().expect("lstsq should succeed");

        assert!(warning.is_some());
        match calc.state().stack.last() {
            Some(Value::Matrix(actual)) => {
                let expected = matrix(2, 1, &[1.0, 2.0]);
                assert_matrix_close(actual, &expected, 1e-10);
            }
            other => panic!("unexpected stack value: {other:?}"),
        }
    }

    #[test]
    fn solve_lstsq_reports_rank_deficiency_warning() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(3, 2, &[1.0, 2.0, 2.0, 4.0, 3.0, 6.0])));
        calc.push_value(Value::Matrix(matrix(3, 1, &[1.0, 2.0, 3.0])));

        let warning = calc.solve_lstsq().expect("lstsq should succeed");

        assert!(warning.is_some());
        assert!(
            warning.expect("warning text").contains("rank-deficient"),
            "expected rank-deficient warning"
        );
    }

    #[test]
    fn inverse_of_singular_matrix_preserves_stack() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, 2.0, 2.0, 4.0])));
        let before = calc.state().stack.clone();

        let result = calc.inverse();

        assert_eq!(
            result,
            Err(CalcError::SingularMatrix(
                "inverse is undefined for singular matrices".to_string()
            ))
        );
        assert_eq!(calc.state().stack, before);
    }

    #[test]
    fn solve_ax_b_dimension_mismatch_preserves_stack() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, 0.0, 0.0, 1.0])));
        calc.push_value(Value::Matrix(matrix(3, 1, &[1.0, 2.0, 3.0])));
        let before = calc.state().stack.clone();

        let result = calc.solve_ax_b();

        assert_eq!(
            result,
            Err(CalcError::DimensionMismatch {
                expected: 2,
                actual: 3
            })
        );
        assert_eq!(calc.state().stack, before);
    }

    #[test]
    fn pow_real_values() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(2.0));
        calc.push_value(Value::Real(3.0));

        let result = calc.pow();

        assert_eq!(result, Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(8.0)]);
    }

    #[test]
    fn percent_real_values() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(200.0));
        calc.push_value(Value::Real(15.0));

        let result = calc.percent();

        assert_eq!(result, Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(30.0)]);
    }

    #[test]
    fn asin_in_degree_mode() {
        let mut calc = Calculator::new();
        calc.set_angle_mode(AngleMode::Deg);
        calc.push_value(Value::Real(0.5));

        let result = calc.asin();

        assert_eq!(result, Ok(()));
        match calc.state().stack.last() {
            Some(Value::Real(v)) => assert_real_close(*v, 30.0, 1e-12),
            other => panic!("unexpected stack value: {other:?}"),
        }
    }

    #[test]
    fn hyperbolic_functions_real_values() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(1.0));
        assert_eq!(calc.sinh(), Ok(()));
        match calc.state().stack.last() {
            Some(Value::Real(v)) => assert_real_close(*v, 1.175_201_193_643_801_4, 1e-12),
            other => panic!("unexpected stack value: {other:?}"),
        }

        calc.push_value(Value::Real(1.0));
        assert_eq!(calc.cosh(), Ok(()));
        match calc.state().stack.last() {
            Some(Value::Real(v)) => assert_real_close(*v, 1.543_080_634_815_243_7, 1e-12),
            other => panic!("unexpected stack value: {other:?}"),
        }
    }

    #[test]
    fn log10_real_value() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(1000.0));

        let result = calc.log10();

        assert_eq!(result, Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(3.0)]);
    }

    #[test]
    fn gamma_and_erf_real_values() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(5.0));
        assert_eq!(calc.gamma(), Ok(()));
        match calc.state().stack.last() {
            Some(Value::Real(v)) => assert_real_close(*v, 24.0, 1e-9),
            other => panic!("unexpected stack value: {other:?}"),
        }

        calc.push_value(Value::Real(1.0));
        assert_eq!(calc.erf(), Ok(()));
        match calc.state().stack.last() {
            Some(Value::Real(v)) => assert_real_close(*v, 0.842_700_79, 1e-6),
            other => panic!("unexpected stack value: {other:?}"),
        }
    }

    #[test]
    fn push_constants() {
        let mut calc = Calculator::new();
        calc.push_pi();
        calc.push_e();

        assert_eq!(calc.state().stack.len(), 2);
        match &calc.state().stack[0] {
            Value::Real(v) => assert_real_close(*v, std::f64::consts::PI, 1e-12),
            other => panic!("unexpected stack value: {other:?}"),
        }
        match &calc.state().stack[1] {
            Value::Real(v) => assert_real_close(*v, std::f64::consts::E, 1e-12),
            other => panic!("unexpected stack value: {other:?}"),
        }
    }

    #[test]
    fn memory_store_recall_and_clear() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(42.0));

        assert_eq!(calc.memory_store(0), Ok(()));
        assert_eq!(calc.clear_all(), ());
        assert_eq!(calc.memory_recall(0), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(42.0)]);
        assert_eq!(calc.memory_clear(0), Ok(()));
        assert_eq!(calc.memory_recall(0), Err(CalcError::EmptyRegister(0)));
    }

    #[test]
    fn memory_invalid_register_error() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(1.0));
        assert_eq!(calc.memory_store(26), Err(CalcError::InvalidRegister(26)));
        assert_eq!(calc.memory_recall(99), Err(CalcError::InvalidRegister(99)));
        assert_eq!(calc.memory_clear(999), Err(CalcError::InvalidRegister(999)));
    }

    #[test]
    fn roll_rotates_top_n_values() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(1.0));
        calc.push_value(Value::Real(2.0));
        calc.push_value(Value::Real(3.0));
        calc.push_value(Value::Real(4.0));

        let result = calc.roll(4);

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![
                Value::Real(2.0),
                Value::Real(3.0),
                Value::Real(4.0),
                Value::Real(1.0)
            ]
        );
    }

    #[test]
    fn pick_duplicates_nth_from_top() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(10.0));
        calc.push_value(Value::Real(20.0));
        calc.push_value(Value::Real(30.0));

        let result = calc.pick(2);

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![
                Value::Real(10.0),
                Value::Real(20.0),
                Value::Real(30.0),
                Value::Real(20.0)
            ]
        );
    }

    #[test]
    fn pick_from_stack_index_replaces_top_value() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(10.0));
        calc.push_value(Value::Real(20.0));
        calc.push_value(Value::Real(30.0));
        calc.push_value(Value::Real(1.0));

        let result = calc.pick_from_stack_index();

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![
                Value::Real(10.0),
                Value::Real(20.0),
                Value::Real(30.0),
                Value::Real(20.0)
            ]
        );
    }

    #[test]
    fn complex_abs_arg_and_conjugate() {
        let mut calc = Calculator::new();
        calc.set_angle_mode(AngleMode::Deg);
        calc.push_value(Value::Complex(Complex { re: 3.0, im: 4.0 }));
        assert_eq!(calc.abs(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(5.0)]);

        calc.clear_all();
        calc.push_value(Value::Complex(Complex { re: 0.0, im: 1.0 }));
        assert_eq!(calc.arg(), Ok(()));
        match calc.state().stack.last() {
            Some(Value::Real(v)) => assert_real_close(*v, 90.0, 1e-12),
            other => panic!("unexpected stack value: {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Complex(Complex { re: -2.0, im: 7.0 }));
        assert_eq!(calc.conjugate(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Complex(Complex { re: -2.0, im: -7.0 })]
        );

        calc.clear_all();
        calc.push_value(Value::Complex(Complex { re: -2.0, im: 7.0 }));
        assert_eq!(calc.real_part(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(-2.0)]);

        calc.clear_all();
        calc.push_value(Value::Complex(Complex { re: -2.0, im: 7.0 }));
        assert_eq!(calc.imag_part(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(7.0)]);

        calc.clear_all();
        calc.push_value(Value::Real(5.0));
        assert_eq!(calc.imag_part(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(0.0)]);
    }

    #[test]
    fn cart_pol_npol_compose_and_decompose() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(3.0));
        calc.push_value(Value::Real(4.0));
        assert_eq!(calc.cart(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Complex(Complex { re: 3.0, im: 4.0 })]
        );

        assert_eq!(calc.cart(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(3.0), Value::Real(4.0)]);

        calc.clear_all();
        calc.set_angle_mode(AngleMode::Deg);
        calc.push_value(Value::Real(2.0));
        calc.push_value(Value::Real(90.0));
        assert_eq!(calc.pol(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Complex(c)] => {
                assert_real_close(c.re, 0.0, 1e-10);
                assert_real_close(c.im, 2.0, 1e-10);
            }
            other => panic!("expected complex output, got {other:?}"),
        }

        assert_eq!(calc.pol(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Real(mag), Value::Real(arg)] => {
                assert_real_close(*mag, 2.0, 1e-10);
                assert_real_close(*arg, 90.0, 1e-10);
            }
            other => panic!("expected mag/arg reals, got {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Real(2.0));
        calc.push_value(Value::Real(0.25));
        assert_eq!(calc.npol(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Complex(c)] => {
                assert_real_close(c.re, 0.0, 1e-10);
                assert_real_close(c.im, 2.0, 1e-10);
            }
            other => panic!("expected complex output, got {other:?}"),
        }

        assert_eq!(calc.npol(), Ok(()));
        match calc.state().stack.as_slice() {
            [Value::Real(mag), Value::Real(cycles)] => {
                assert_real_close(*mag, 2.0, 1e-10);
                assert_real_close(*cycles, 0.25, 1e-10);
            }
            other => panic!("expected mag/cycles reals, got {other:?}"),
        }
    }

    #[test]
    fn root_and_log2_exp2() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(27.0));
        calc.push_value(Value::Real(3.0));
        assert_eq!(calc.root(), Ok(()));
        match calc.state().stack.last() {
            Some(Value::Real(v)) => assert_real_close(*v, 3.0, 1e-12),
            other => panic!("unexpected stack value: {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Real(8.0));
        assert_eq!(calc.log2(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(3.0)]);

        calc.clear_all();
        calc.push_value(Value::Real(5.0));
        assert_eq!(calc.exp2(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(32.0)]);
    }

    #[test]
    fn factorial_combinations_and_integer_ops() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(5.0));
        assert_eq!(calc.factorial(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(120.0)]);

        calc.clear_all();
        calc.push_value(Value::Real(5.0));
        calc.push_value(Value::Real(2.0));
        assert_eq!(calc.ncr(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(10.0)]);

        calc.clear_all();
        calc.push_value(Value::Real(5.0));
        calc.push_value(Value::Real(2.0));
        assert_eq!(calc.npr(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(20.0)]);

        calc.clear_all();
        calc.push_value(Value::Real(42.0));
        calc.push_value(Value::Real(30.0));
        assert_eq!(calc.gcd(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(6.0)]);

        calc.clear_all();
        calc.push_value(Value::Real(12.0));
        calc.push_value(Value::Real(18.0));
        assert_eq!(calc.lcm(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(36.0)]);
    }
