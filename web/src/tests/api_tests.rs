    use super::{ApiAngleMode, ApiDisplayMode, ApiValue, CalculatorApi, ComplexInput, MatrixInput};

    fn c(re: f64, im: f64) -> ComplexInput {
        ComplexInput { re, im }
    }

    #[test]
    fn successful_operation_returns_ok_with_updated_state() {
        let mut api = CalculatorApi::new();
        api.push_real(2.0);
        api.push_real(3.0);

        let response = api.add();

        assert!(response.ok);
        assert_eq!(response.error, None);
        assert_eq!(response.state.stack, vec![ApiValue::Real { value: 5.0 }]);
    }

    #[test]
    fn undo_restores_previous_successful_state() {
        let mut api = CalculatorApi::new();
        api.push_real(2.0);
        api.push_real(3.0);
        let add = api.add();
        assert!(add.ok);
        assert_eq!(add.state.stack, vec![ApiValue::Real { value: 5.0 }]);

        let undo = api.undo();
        assert!(undo.ok);
        assert_eq!(
            undo.state.stack,
            vec![ApiValue::Real { value: 2.0 }, ApiValue::Real { value: 3.0 }]
        );

        let second_undo = api.undo();
        assert!(!second_undo.ok);
        let error = second_undo.error.expect("undo error expected");
        assert_eq!(error.code, "invalid_input");
    }

    #[test]
    fn complex_real_and_imag_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_complex(ComplexInput { re: -2.0, im: 7.0 });

        let real_response = api.real_part();
        assert!(real_response.ok);
        assert_eq!(
            real_response.state.stack,
            vec![ApiValue::Real { value: -2.0 }]
        );

        api.clear_all();
        api.push_complex(ComplexInput { re: -2.0, im: 7.0 });

        let imag_response = api.imag_part();
        assert!(imag_response.ok);
        assert_eq!(
            imag_response.state.stack,
            vec![ApiValue::Real { value: 7.0 }]
        );
    }

    #[test]
    fn cart_pol_npol_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_real(3.0);
        api.push_real(4.0);
        let cart_response = api.cart();
        assert!(cart_response.ok);
        assert_eq!(
            cart_response.state.stack,
            vec![ApiValue::Complex { re: 3.0, im: 4.0 }]
        );

        let cart_back = api.cart();
        assert!(cart_back.ok);
        assert_eq!(
            cart_back.state.stack,
            vec![ApiValue::Real { value: 3.0 }, ApiValue::Real { value: 4.0 }]
        );

        api.clear_all();
        api.set_angle_mode(ApiAngleMode::Deg);
        api.push_real(2.0);
        api.push_real(90.0);
        let pol_response = api.pol();
        assert!(pol_response.ok);
        match pol_response.state.stack.as_slice() {
            [ApiValue::Complex { re, im }] => {
                assert!(re.abs() < 1e-10);
                assert!((im - 2.0).abs() < 1e-10);
            }
            other => panic!("expected complex output, got {other:?}"),
        }

        let pol_back = api.pol();
        assert!(pol_back.ok);
        match pol_back.state.stack.as_slice() {
            [ApiValue::Real { value: mag }, ApiValue::Real { value: arg }] => {
                assert!((mag - 2.0).abs() < 1e-10);
                assert!((arg - 90.0).abs() < 1e-10);
            }
            other => panic!("expected mag/arg output, got {other:?}"),
        }

        api.clear_all();
        api.push_real(2.0);
        api.push_real(0.25);
        let npol_response = api.npol();
        assert!(npol_response.ok);
        match npol_response.state.stack.as_slice() {
            [ApiValue::Complex { re, im }] => {
                assert!(re.abs() < 1e-10);
                assert!((im - 2.0).abs() < 1e-10);
            }
            other => panic!("expected complex output, got {other:?}"),
        }
    }

    #[test]
    fn hadamard_ops_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 3,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0)],
        });
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 3,
            data: vec![c(4.0, 0.0), c(5.0, 0.0), c(6.0, 0.0)],
        });

        let mul_response = api.hadamard_mul();
        assert!(mul_response.ok);
        assert_eq!(mul_response.state.stack.len(), 1);

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 3,
            data: vec![c(8.0, 0.0), c(10.0, 0.0), c(18.0, 0.0)],
        });
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 3,
            data: vec![c(2.0, 0.0), c(5.0, 0.0), c(3.0, 0.0)],
        });

        let div_response = api.hadamard_div();
        assert!(div_response.ok);
        assert_eq!(div_response.state.stack.len(), 1);

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 3,
            data: vec![c(2.0, 0.0), c(4.0, 0.0), c(8.0, 0.0)],
        });
        api.push_real(2.0);

        let scalar_mul_response = api.hadamard_mul();
        assert!(scalar_mul_response.ok);
        assert_eq!(
            scalar_mul_response.state.stack,
            vec![ApiValue::Matrix {
                rows: 1,
                cols: 3,
                data: vec![c(4.0, 0.0), c(8.0, 0.0), c(16.0, 0.0)]
            }]
        );

        api.clear_all();
        api.push_real(8.0);
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 3,
            data: vec![c(2.0, 0.0), c(4.0, 0.0), c(8.0, 0.0)],
        });

        let scalar_div_response = api.hadamard_div();
        assert!(scalar_div_response.ok);
        assert_eq!(
            scalar_div_response.state.stack,
            vec![ApiValue::Matrix {
                rows: 1,
                cols: 3,
                data: vec![c(4.0, 0.0), c(2.0, 0.0), c(1.0, 0.0)]
            }]
        );
    }

    #[test]
    fn failing_operation_returns_error_and_preserved_state() {
        let mut api = CalculatorApi::new();
        api.push_real(4.0);
        api.push_real(0.0);

        let response = api.div();

        assert!(!response.ok);
        let error = response.error.expect("error expected");
        assert_eq!(error.code, "divide_by_zero");
        assert_eq!(
            response.state.stack,
            vec![ApiValue::Real { value: 4.0 }, ApiValue::Real { value: 0.0 }]
        );
    }

    #[test]
    fn snapshot_contains_mode_and_entry_state() {
        let mut api = CalculatorApi::new();
        api.entry_set("90");
        api.set_angle_mode(ApiAngleMode::Deg);

        let snapshot = api.snapshot();

        assert_eq!(snapshot.entry_buffer, "90");
        assert_eq!(snapshot.angle_mode, ApiAngleMode::Deg);
    }

    #[test]
    fn display_preferences_can_be_updated_via_api() {
        let mut api = CalculatorApi::new();

        let display_response = api.set_display_mode(ApiDisplayMode::Sci);
        assert!(display_response.ok);
        assert_eq!(display_response.state.display_mode, ApiDisplayMode::Sci);

        let precision_response = api.set_precision(9);
        assert!(precision_response.ok);
        assert_eq!(precision_response.state.precision, 9);
    }

    #[test]
    fn batch_script_executes_against_calculator_state() {
        let mut api = CalculatorApi::new();

        let response = api.run_script(
            r#"
            2 3 add
            4 mul
            "#,
        );

        assert!(response.ok);
        assert_eq!(response.state.stack, vec![ApiValue::Real { value: 20.0 }]);
        assert!(response.transcript.len() >= 4);
    }

    #[test]
    fn interactive_script_line_updates_state() {
        let mut api = CalculatorApi::new();

        let response = api.run_script_line("9 3 div");

        assert!(response.ok);
        assert_eq!(response.state.stack, vec![ApiValue::Real { value: 3.0 }]);
    }

    #[test]
    fn script_can_use_memory_registers_as_variables() {
        let mut api = CalculatorApi::new();

        let response = api.run_script(
            r#"
            12 store A
            clear_all
            recall A
            8 add
            "#,
        );

        assert!(response.ok);
        assert_eq!(response.state.stack, vec![ApiValue::Real { value: 20.0 }]);
    }

    #[test]
    fn failed_script_preserves_original_state() {
        let mut api = CalculatorApi::new();
        api.push_real(7.0);

        let response = api.run_script(
            r#"
            5 add
            unknown_command
            "#,
        );

        assert!(!response.ok);
        assert_eq!(response.state.stack, vec![ApiValue::Real { value: 7.0 }]);
        let error = response.error.expect("script error expected");
        assert_eq!(error.code, "unknown_command");
        assert_eq!(error.line, 3);
    }

    #[test]
    fn push_matrix_adds_matrix_to_stack() {
        let mut api = CalculatorApi::new();
        let matrix = MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)],
        };

        let response = api.push_matrix(matrix);

        assert!(response.ok);
        assert_eq!(
            response.state.stack,
            vec![ApiValue::Matrix {
                rows: 2,
                cols: 2,
                data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)]
            }]
        );
    }

    #[test]
    fn push_complex_adds_complex_to_stack() {
        let mut api = CalculatorApi::new();

        let response = api.push_complex(ComplexInput { re: 1.5, im: -2.0 });

        assert!(response.ok);
        assert_eq!(
            response.state.stack,
            vec![ApiValue::Complex { re: 1.5, im: -2.0 }]
        );
    }

    #[test]
    fn push_identity_adds_identity_matrix() {
        let mut api = CalculatorApi::new();

        let response = api.push_identity(2);

        assert!(response.ok);
        assert_eq!(
            response.state.stack,
            vec![ApiValue::Matrix {
                rows: 2,
                cols: 2,
                data: vec![c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(1.0, 0.0)]
            }]
        );
    }

    #[test]
    fn stack_vec_converts_stack_scalars_to_matrix() {
        let mut api = CalculatorApi::new();
        api.push_real(1.0);
        api.push_complex(ComplexInput { re: 2.0, im: -1.0 });
        api.push_real(3.5);

        let response = api.stack_vec();

        assert!(response.ok);
        assert_eq!(
            response.state.stack,
            vec![ApiValue::Matrix {
                rows: 3,
                cols: 1,
                data: vec![c(1.0, 0.0), c(2.0, -1.0), c(3.5, 0.0)]
            }]
        );
    }

    #[test]
    fn ravel_matrix_and_vector_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)],
        });

        let matrix_response = api.ravel();
        assert!(matrix_response.ok);
        assert_eq!(
            matrix_response.state.stack,
            vec![ApiValue::Matrix {
                rows: 4,
                cols: 1,
                data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)]
            }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 3,
            data: vec![c(1.0, 0.0), c(2.0, -1.0), c(3.0, 0.0)],
        });

        let vector_response = api.ravel();
        assert!(vector_response.ok);
        assert_eq!(
            vector_response.state.stack,
            vec![
                ApiValue::Real { value: 1.0 },
                ApiValue::Complex { re: 2.0, im: -1.0 },
                ApiValue::Real { value: 3.0 },
            ]
        );
    }

    #[test]
    fn hstack_vstack_and_split_ravel_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_real(1.0);
        api.push_real(2.0);
        api.push_real(3.0);
        api.push_real(3.0);
        let hstack_response = api.hstack();
        assert!(hstack_response.ok);
        assert_eq!(
            hstack_response.state.stack,
            vec![ApiValue::Matrix {
                rows: 1,
                cols: 3,
                data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0)]
            }]
        );

        api.clear_all();
        api.push_real(1.0);
        api.push_real(2.0);
        api.push_real(3.0);
        api.push_real(3.0);
        let vstack_response = api.vstack();
        assert!(vstack_response.ok);
        assert_eq!(
            vstack_response.state.stack,
            vec![ApiValue::Matrix {
                rows: 3,
                cols: 1,
                data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0)]
            }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 3,
            data: vec![
                c(1.0, 0.0),
                c(2.0, 0.0),
                c(3.0, 0.0),
                c(4.0, 0.0),
                c(5.0, 0.0),
                c(6.0, 0.0),
            ],
        });
        let hravel_response = api.hravel();
        assert!(hravel_response.ok);
        assert_eq!(hravel_response.state.stack.len(), 3);

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 3,
            data: vec![
                c(1.0, 0.0),
                c(2.0, 0.0),
                c(3.0, 0.0),
                c(4.0, 0.0),
                c(5.0, 0.0),
                c(6.0, 0.0),
            ],
        });
        let vravel_response = api.vravel();
        assert!(vravel_response.ok);
        assert_eq!(vravel_response.state.stack.len(), 2);
    }

    #[test]
    fn dot_trace_and_norm_p_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 3,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0)],
        });
        api.push_matrix(MatrixInput {
            rows: 3,
            cols: 1,
            data: vec![c(4.0, 0.0), c(5.0, 0.0), c(6.0, 0.0)],
        });

        let dot_response = api.dot();
        assert!(dot_response.ok);
        assert_eq!(
            dot_response.state.stack,
            vec![ApiValue::Complex { re: 32.0, im: 0.0 }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)],
        });

        let trace_response = api.trace();
        assert!(trace_response.ok);
        assert_eq!(
            trace_response.state.stack,
            vec![ApiValue::Complex { re: 5.0, im: 0.0 }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 2,
            data: vec![c(3.0, 0.0), c(4.0, 0.0)],
        });
        api.push_real(2.0);

        let norm_response = api.norm_p();
        assert!(norm_response.ok);
        assert_eq!(
            norm_response.state.stack,
            vec![ApiValue::Real { value: 5.0 }]
        );
    }

    #[test]
    fn solve_lstsq_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 3,
            cols: 2,
            data: vec![
                c(1.0, 0.0),
                c(0.0, 0.0),
                c(0.0, 0.0),
                c(1.0, 0.0),
                c(1.0, 0.0),
                c(1.0, 0.0),
            ],
        });
        api.push_matrix(MatrixInput {
            rows: 3,
            cols: 1,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0)],
        });

        let response = api.solve_lstsq();
        assert!(response.ok);
        assert!(response.warning.is_some());
        assert!(
            response
                .warning
                .as_ref()
                .expect("warning text")
                .contains("residual norm")
        );
        match response.state.stack.as_slice() {
            [ApiValue::Matrix { rows, cols, data }] => {
                assert_eq!((*rows, *cols), (2, 1));
                assert!((data[0].re - 1.0).abs() < 1e-10);
                assert!((data[1].re - 2.0).abs() < 1e-10);
            }
            other => panic!("expected matrix response, got {other:?}"),
        }
    }

    #[test]
    fn diag_and_mat_exp_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 3,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0)],
        });

        let diag_response = api.diag();
        assert!(diag_response.ok);
        assert_eq!(
            diag_response.state.stack,
            vec![ApiValue::Matrix {
                rows: 3,
                cols: 3,
                data: vec![
                    c(1.0, 0.0),
                    c(0.0, 0.0),
                    c(0.0, 0.0),
                    c(0.0, 0.0),
                    c(2.0, 0.0),
                    c(0.0, 0.0),
                    c(0.0, 0.0),
                    c(0.0, 0.0),
                    c(3.0, 0.0),
                ]
            }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 3,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0)],
        });

        let toep_response = api.toep();
        assert!(toep_response.ok);
        assert_eq!(
            toep_response.state.stack,
            vec![ApiValue::Matrix {
                rows: 3,
                cols: 3,
                data: vec![
                    c(1.0, 0.0),
                    c(2.0, 0.0),
                    c(3.0, 0.0),
                    c(2.0, 0.0),
                    c(1.0, 0.0),
                    c(2.0, 0.0),
                    c(3.0, 0.0),
                    c(2.0, 0.0),
                    c(1.0, 0.0),
                ]
            }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(2.0, 0.0)],
        });

        let exp_response = api.mat_exp();
        assert!(exp_response.ok);
        match exp_response.state.stack.as_slice() {
            [ApiValue::Matrix { rows, cols, data }] => {
                assert_eq!((*rows, *cols), (2, 2));
                assert!((data[0].re - std::f64::consts::E).abs() < 1e-10);
                assert!(data[1].re.abs() < 1e-10);
                assert!(data[2].re.abs() < 1e-10);
                assert!((data[3].re - std::f64::consts::E.powi(2)).abs() < 1e-10);
            }
            other => panic!("expected matrix response, got {other:?}"),
        }
    }

    #[test]
    fn scalar_complex_rounding_matrix_elementwise_via_api() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 3,
            data: vec![c(3.0, 0.0), c(4.0, 0.0), c(5.0, 0.0)],
        });
        api.push_real(2.0);
        let pow_response = api.pow();
        assert!(pow_response.ok);
        match pow_response.state.stack.as_slice() {
            [ApiValue::Matrix { rows, cols, data }] => {
                assert_eq!((*rows, *cols), (1, 3));
                assert!((data[0].re - 9.0).abs() < 1e-12);
                assert!((data[1].re - 16.0).abs() < 1e-12);
                assert!((data[2].re - 25.0).abs() < 1e-12);
            }
            other => panic!("expected matrix after elementwise pow, got {other:?}"),
        }

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 3,
            data: vec![c(1.2, 0.0), c(-2.5, 0.0), c(3.8, 0.0)],
        });
        let round_response = api.round_value();
        assert!(round_response.ok);
        match round_response.state.stack.as_slice() {
            [ApiValue::Matrix { rows, cols, data }] => {
                assert_eq!((*rows, *cols), (1, 3));
                assert!((data[0].re - 1.0).abs() < 1e-12);
                assert!((data[1].re + 3.0).abs() < 1e-12);
                assert!((data[2].re - 4.0).abs() < 1e-12);
            }
            other => panic!("expected matrix after elementwise round, got {other:?}"),
        }
    }

    #[test]
    fn hermitian_and_mat_pow_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(1.0, 2.0), c(3.0, -1.0), c(-4.0, 0.5), c(2.0, 0.0)],
        });

        let herm_response = api.hermitian();
        assert!(herm_response.ok);
        assert_eq!(herm_response.state.stack.len(), 1);

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(2.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(3.0, 0.0)],
        });
        api.push_real(3.0);

        let pow_response = api.mat_pow();
        assert!(pow_response.ok);
        match pow_response.state.stack.as_slice() {
            [ApiValue::Matrix { rows, cols, data }] => {
                assert_eq!((*rows, *cols), (2, 2));
                assert!((data[0].re - 8.0).abs() < 1e-12);
                assert!((data[3].re - 27.0).abs() < 1e-12);
            }
            other => panic!("expected matrix mat_pow output, got {other:?}"),
        }
    }

    #[test]
    fn qr_and_lu_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)],
        });

        let qr_response = api.qr();
        assert!(qr_response.ok);
        assert_eq!(qr_response.state.stack.len(), 2);

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(4.0, 0.0), c(3.0, 0.0), c(6.0, 0.0), c(3.0, 0.0)],
        });

        let lu_response = api.lu();
        assert!(lu_response.ok);
        assert_eq!(lu_response.state.stack.len(), 3);

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(1.0, 1.0), c(2.0, -0.5), c(0.5, 0.0), c(3.0, 2.0)],
        });
        let complex_qr = api.qr();
        assert!(complex_qr.ok);
        assert_eq!(complex_qr.state.stack.len(), 2);

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(1.0, 1.0), c(2.0, -0.5), c(0.5, 0.0), c(3.0, 2.0)],
        });
        let complex_lu = api.lu();
        assert!(complex_lu.ok);
        assert_eq!(complex_lu.state.stack.len(), 3);
    }

    #[test]
    fn svd_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(3.0, 0.0), c(1.0, 0.0), c(1.0, 0.0), c(3.0, 0.0)],
        });

        let response = api.svd();
        assert!(response.ok);
        assert_eq!(response.state.stack.len(), 3);
        match response.state.stack.as_slice() {
            [
                ApiValue::Matrix {
                    rows: ur, cols: uc, ..
                },
                ApiValue::Matrix {
                    rows: sr, cols: sc, ..
                },
                ApiValue::Matrix {
                    rows: vr, cols: vc, ..
                },
            ] => {
                assert_eq!((*ur, *uc), (2, 2));
                assert_eq!((*sr, *sc), (2, 2));
                assert_eq!((*vr, *vc), (2, 2));
            }
            other => panic!("expected three matrices from svd, got {other:?}"),
        }

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(1.0, 2.0), c(0.0, -1.0), c(3.0, 0.5), c(-2.0, 0.0)],
        });

        let complex_response = api.svd();
        assert!(complex_response.ok);
        assert_eq!(complex_response.state.stack.len(), 3);
    }

    #[test]
    fn evd_work_via_api_with_warning() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(2.0, 1.0), c(0.0, 0.0), c(0.0, 0.0), c(-1.0, 0.5)],
        });

        let exact_response = api.evd();
        assert!(exact_response.ok);
        assert_eq!(exact_response.state.stack.len(), 2);
        assert!(exact_response.warning.is_none());

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(1.0, 0.0), c(1.0, 0.0), c(0.0, 0.0), c(1.0, 0.0)],
        });

        let response = api.evd();
        assert!(response.ok);
        assert_eq!(response.state.stack.len(), 2);
        assert!(response.warning.is_some());
    }

    #[test]
    fn vector_statistics_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 5,
            data: vec![
                c(1.0, 0.0),
                c(2.0, 0.0),
                c(2.0, 0.0),
                c(4.0, 0.0),
                c(5.0, 0.0),
            ],
        });

        let mean_response = api.mean();
        assert!(mean_response.ok);
        assert_eq!(
            mean_response.state.stack,
            vec![ApiValue::Real { value: 2.8 }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 5,
            data: vec![
                c(1.0, 0.0),
                c(2.0, 0.0),
                c(2.0, 0.0),
                c(4.0, 0.0),
                c(5.0, 0.0),
            ],
        });

        let mode_response = api.mode();
        assert!(mode_response.ok);
        assert_eq!(
            mode_response.state.stack,
            vec![ApiValue::Real { value: 2.0 }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 2,
            data: vec![c(3.0, 0.0), c(4.0, 0.0)],
        });

        let variance_response = api.variance();
        assert!(variance_response.ok);
        assert_eq!(
            variance_response.state.stack,
            vec![ApiValue::Real { value: 0.25 }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 2,
            data: vec![c(3.0, 0.0), c(4.0, 0.0)],
        });

        let std_response = api.std_dev_p();
        assert!(std_response.ok);
        assert_eq!(
            std_response.state.stack,
            vec![ApiValue::Real { value: 0.5 }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 5,
            data: vec![
                c(1.0, 0.0),
                c(2.0, 0.0),
                c(2.0, 0.0),
                c(4.0, 0.0),
                c(5.0, 0.0),
            ],
        });

        let max_response = api.max_value();
        assert!(max_response.ok);
        assert_eq!(
            max_response.state.stack,
            vec![ApiValue::Real { value: 5.0 }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 5,
            data: vec![
                c(1.0, 0.0),
                c(2.0, 0.0),
                c(2.0, 0.0),
                c(4.0, 0.0),
                c(5.0, 0.0),
            ],
        });

        let min_response = api.min_value();
        assert!(min_response.ok);
        assert_eq!(
            min_response.state.stack,
            vec![ApiValue::Real { value: 1.0 }]
        );

        api.clear_all();
        api.push_real(1.0);
        api.push_real(2.0);
        api.push_real(5.0);

        let scalar_mean_response = api.mean();
        assert!(scalar_mean_response.ok);
        assert_eq!(
            scalar_mean_response.state.stack,
            vec![ApiValue::Real { value: 8.0 / 3.0 }]
        );
    }

    #[test]
    fn stack_utility_operations_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_real(1.0);
        api.push_real(2.0);
        api.push_real(3.0);

        let swap_response = api.swap();
        assert!(swap_response.ok);
        assert_eq!(
            swap_response.state.stack,
            vec![
                ApiValue::Real { value: 1.0 },
                ApiValue::Real { value: 3.0 },
                ApiValue::Real { value: 2.0 }
            ]
        );

        let dup_response = api.dup();
        assert!(dup_response.ok);
        assert_eq!(
            dup_response.state.stack,
            vec![
                ApiValue::Real { value: 1.0 },
                ApiValue::Real { value: 3.0 },
                ApiValue::Real { value: 2.0 },
                ApiValue::Real { value: 2.0 }
            ]
        );

        let drop_response = api.drop();
        assert!(drop_response.ok);
        assert_eq!(
            drop_response.state.stack,
            vec![
                ApiValue::Real { value: 1.0 },
                ApiValue::Real { value: 3.0 },
                ApiValue::Real { value: 2.0 }
            ]
        );
    }

    #[test]
    fn roll_and_pick_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_real(1.0);
        api.push_real(2.0);
        api.push_real(3.0);
        api.push_real(4.0);

        let roll_response = api.roll(4);
        assert!(roll_response.ok);
        assert_eq!(
            roll_response.state.stack,
            vec![
                ApiValue::Real { value: 2.0 },
                ApiValue::Real { value: 3.0 },
                ApiValue::Real { value: 4.0 },
                ApiValue::Real { value: 1.0 }
            ]
        );

        let pick_response = api.pick(2);
        assert!(pick_response.ok);
        assert_eq!(
            pick_response.state.stack,
            vec![
                ApiValue::Real { value: 2.0 },
                ApiValue::Real { value: 3.0 },
                ApiValue::Real { value: 4.0 },
                ApiValue::Real { value: 1.0 },
                ApiValue::Real { value: 4.0 }
            ]
        );

        api.push_real(1.0);
        let pick_index_response = api.pick_from_stack_index();
        assert!(pick_index_response.ok);
        assert_eq!(
            pick_index_response.state.stack,
            vec![
                ApiValue::Real { value: 2.0 },
                ApiValue::Real { value: 3.0 },
                ApiValue::Real { value: 4.0 },
                ApiValue::Real { value: 1.0 },
                ApiValue::Real { value: 4.0 },
                ApiValue::Real { value: 3.0 }
            ]
        );
    }

    #[test]
    fn scalar_extensions_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_real(2.0);
        api.push_real(8.0);
        let log_response = api.log_y_x();
        assert!(log_response.ok);
        assert_eq!(
            log_response.state.stack,
            vec![ApiValue::Real { value: 3.0 }]
        );

        let neg_response = api.neg();
        assert!(neg_response.ok);
        assert_eq!(
            neg_response.state.stack,
            vec![ApiValue::Real { value: -3.0 }]
        );

        api.clear_all();
        api.push_real(0.0);
        let sinc_response = api.sinc();
        assert!(sinc_response.ok);
        assert_eq!(
            sinc_response.state.stack,
            vec![ApiValue::Real { value: 1.0 }]
        );

        api.clear_all();
        api.push_real(0.0);
        let erfc_response = api.erfc();
        assert!(erfc_response.ok);
        match erfc_response.state.stack.as_slice() {
            [ApiValue::Real { value }] => {
                assert!((*value - 1.0).abs() < 1e-8);
            }
            other => panic!("expected real erfc result, got {other:?}"),
        }
    }

    #[test]
    fn reciprocal_trig_works_via_api() {
        let mut api = CalculatorApi::new();
        api.push_real(0.0);
        let sec_response = api.sec();
        assert!(sec_response.ok);
        assert_eq!(
            sec_response.state.stack,
            vec![ApiValue::Real { value: 1.0 }]
        );

        api.clear_all();
        api.push_real(1.0);
        let acot_response = api.acot();
        assert!(acot_response.ok);
        match acot_response.state.stack.as_slice() {
            [ApiValue::Real { value }] => {
                assert!((*value - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
            }
            other => panic!("expected real acot result, got {other:?}"),
        }
    }

    #[test]
    fn statistics_extensions_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 4,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)],
        });
        let median_response = api.median();
        assert!(median_response.ok);
        assert_eq!(
            median_response.state.stack,
            vec![ApiValue::Real { value: 2.5 }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 4,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)],
        });
        let quart_response = api.quart();
        assert!(quart_response.ok);
        match quart_response.state.stack.as_slice() {
            [ApiValue::Matrix { rows, cols, data }] => {
                assert_eq!((*rows, *cols), (1, 5));
                assert!((data[0].re - 1.0).abs() < 1e-10);
                assert!((data[1].re - 1.75).abs() < 1e-10);
                assert!((data[2].re - 2.5).abs() < 1e-10);
                assert!((data[3].re - 3.25).abs() < 1e-10);
                assert!((data[4].re - 4.0).abs() < 1e-10);
            }
            other => panic!("expected quartile matrix, got {other:?}"),
        }

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 4,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)],
        });
        let std_p_response = api.std_dev_p();
        assert!(std_p_response.ok);
        match std_p_response.state.stack.as_slice() {
            [ApiValue::Real { value }] => {
                assert!((*value - 1.118033988749895).abs() < 1e-10);
            }
            other => panic!("expected population std-dev scalar, got {other:?}"),
        }

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 4,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)],
        });
        let std_s_response = api.std_dev_s();
        assert!(std_s_response.ok);
        match std_s_response.state.stack.as_slice() {
            [ApiValue::Real { value }] => {
                assert!((*value - 1.2909944487358056).abs() < 1e-10);
            }
            other => panic!("expected sample std-dev scalar, got {other:?}"),
        }
    }
