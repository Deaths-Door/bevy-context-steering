use bevy_context_steering::Falloff;
use test_case::test_case;

const T: f32 = 10.0;
const EPS: f32 = 1e-4;

fn thresholded_variants() -> Vec<Falloff> {
    vec![
        Falloff::Linear { threshold: T },
        Falloff::Quadratic { threshold: T },
        Falloff::Cubic { threshold: T },
        Falloff::SmoothStep { threshold: T },
        Falloff::SmootherStep { threshold: T },
        Falloff::InverseSquare { threshold: T },
        Falloff::Exponential {
            threshold: T,
            exponent: 2.0,
        },
    ]
}

// ---------- Falloff::None ----------

#[test]
fn none_has_no_threshold() {
    assert_eq!(Falloff::None.threshold(), None);
}

#[test]
fn none_always_full_factor_both_directions() {
    for d in [0.0, 0.001, 5.0, 10.0, 1000.0] {
        assert_eq!(Falloff::None.inwards_factor(d), 1.0, "inwards @ d={d}");
        assert_eq!(Falloff::None.outwards_factor(d), 1.0, "outwards @ d={d}");
    }
}

// ---------- threshold() accessor ----------

#[test_case(Falloff::Linear { threshold: T })]
#[test_case(Falloff::Quadratic { threshold: T })]
#[test_case(Falloff::Cubic { threshold: T })]
#[test_case(Falloff::SmoothStep { threshold: T })]
#[test_case(Falloff::SmootherStep { threshold: T })]
#[test_case(Falloff::InverseSquare { threshold: T })]
#[test_case(Falloff::Exponential { threshold: T, exponent: 2.0 })]
fn threshold_returns_configured_value(falloff: Falloff) {
    assert_eq!(falloff.threshold(), Some(T));
}

// ---------- inwards_factor boundary contract ----------
// 0.0 at center, ramps to 1.0 at/after threshold.

#[test]
fn inwards_zero_at_center_all_variants() {
    for f in thresholded_variants() {
        let v = f.inwards_factor(0.0);
        assert!(
            v.abs() < EPS,
            "{f:?} inwards_factor(0.0) should be ~0, got {v}"
        );
    }
}

#[test]
fn inwards_one_at_and_beyond_threshold_all_variants() {
    for f in thresholded_variants() {
        for d in [T, T + 0.001, T * 2.0, T * 100.0] {
            let v = f.inwards_factor(d);
            assert!(
                (v - 1.0).abs() < EPS,
                "{f:?} inwards_factor({d}) should be ~1, got {v}"
            );
        }
    }
}

#[test]
fn inwards_stays_in_unit_range() {
    for f in thresholded_variants() {
        let mut d = 0.0;
        while d <= T * 1.5 {
            let v = f.inwards_factor(d);
            assert!(
                (0.0..=1.0 + EPS).contains(&v) && !v.is_nan(),
                "{f:?} out of range at d={d}: {v}"
            );
            d += 0.1;
        }
    }
}

#[test]
fn inwards_monotonic_nondecreasing_with_distance() {
    for f in thresholded_variants() {
        let samples: Vec<f32> = (0..=100).map(|i| i as f32 / 100.0 * T).collect();
        let mut prev = f.inwards_factor(samples[0]);
        for &d in &samples[1..] {
            let v = f.inwards_factor(d);
            assert!(
                v >= prev - EPS,
                "{f:?} inwards not monotonic at d={d}: {v} < prev {prev}"
            );
            prev = v;
        }
    }
}

// ---------- outwards_factor boundary contract ----------
// 0.0 at/beyond threshold (out of range), ramps to 1.0 at center.
// Note the strict `<` in the source: distance == threshold hits the
// `distance < threshold` false branch, so it returns 0.0, not curve_factor(0.0).

#[test]
fn outwards_zero_at_and_beyond_threshold_all_variants() {
    for f in thresholded_variants() {
        for d in [T, T + 0.001, T * 2.0, T * 100.0] {
            let v = f.outwards_factor(d);
            assert!(
                v.abs() < EPS,
                "{f:?} outwards_factor({d}) should be ~0, got {v}"
            );
        }
    }
}

#[test]
fn outwards_one_at_center_all_variants() {
    for f in thresholded_variants() {
        let v = f.outwards_factor(0.0);
        assert!(
            (v - 1.0).abs() < EPS,
            "{f:?} outwards_factor(0.0) should be ~1, got {v}"
        );
    }
}

#[test]
fn outwards_stays_in_unit_range() {
    for f in thresholded_variants() {
        let mut d = 0.0;
        while d <= T * 1.5 {
            let v = f.outwards_factor(d);
            assert!(
                (0.0..=1.0 + EPS).contains(&v) && !v.is_nan(),
                "{f:?} out of range at d={d}: {v}"
            );
            d += 0.1;
        }
    }
}

#[test]
fn outwards_monotonic_nonincreasing_with_distance() {
    for f in thresholded_variants() {
        let samples: Vec<f32> = (0..=100).map(|i| i as f32 / 100.0 * T).collect();
        let mut prev = f.outwards_factor(samples[0]);
        for &d in &samples[1..] {
            let v = f.outwards_factor(d);
            assert!(
                v <= prev + EPS,
                "{f:?} outwards not monotonic at d={d}: {v} > prev {prev}"
            );
            prev = v;
        }
    }
}

// ---------- inwards/outwards mirror relationship ----------
// outwards_factor(d) should equal inwards_factor(threshold - d) for d in (0, threshold),
// since both funnel through the same curve_factor with x flipped.
// Excludes d=0 and d=threshold because outwards has a hard 0 cutoff at
// the boundary (strict `<`) that inwards doesn't mirror exactly there.

#[test]
fn outwards_mirrors_inwards_through_curve_factor() {
    for f in thresholded_variants() {
        for frac in [0.1, 0.25, 0.4, 0.6, 0.75, 0.9] {
            let d = frac * T;
            let out = f.outwards_factor(d);
            let mirrored_in = f.inwards_factor(T - d);
            assert!(
                (out - mirrored_in).abs() < EPS,
                "{f:?} mirror mismatch at d={d}: outwards={out} inwards(T-d)={mirrored_in}"
            );
        }
    }
}

// ---------- Negative / degenerate distance safety ----------

#[test]
fn negative_distance_does_not_panic_or_nan() {
    for f in thresholded_variants() {
        for d in [-0.001, -5.0, -100.0] {
            let vi = f.inwards_factor(d);
            let vo = f.outwards_factor(d);
            assert!(
                !vi.is_nan() && vi >= 0.0 && vi <= 1.0,
                "{f:?} inwards NaN/out-of-range at d={d}: {vi}"
            );
            assert!(
                !vo.is_nan() && vo >= 0.0 && vo <= 1.0,
                "{f:?} outwards NaN/out-of-range at d={d}: {vo}"
            );
        }
    }
}

#[test_case(Falloff::Linear { threshold: 0.0 })]
#[test_case(Falloff::Quadratic { threshold: 0.0 })]
#[test_case(Falloff::SmoothStep { threshold: 0.0 })]
fn zero_threshold_does_not_panic_or_nan(falloff: Falloff) {
    // Guards the `distance / threshold` division inside inwards/outwards_factor.
    for d in [0.0, 1.0, 10.0] {
        assert!(
            !falloff.inwards_factor(d).is_nan(),
            "{falloff:?} inwards NaN at d={d}, threshold=0"
        );
        assert!(
            !falloff.outwards_factor(d).is_nan(),
            "{falloff:?} outwards NaN at d={d}, threshold=0"
        );
    }
}

// ---------- Shape-specific checks (via inwards_factor, x = d/threshold) ----------

#[test]
fn linear_is_exactly_proportional() {
    let f = Falloff::Linear { threshold: T };
    for frac in [0.0, 0.25, 0.5, 0.75, 1.0] {
        assert!(
            (f.inwards_factor(frac * T) - frac).abs() < EPS,
            "frac={frac}"
        );
    }
}

#[test]
fn quadratic_lags_behind_linear() {
    let quad = Falloff::Quadratic { threshold: T };
    let lin = Falloff::Linear { threshold: T };
    for frac in [0.1, 0.25, 0.4, 0.6, 0.75, 0.9] {
        let d = frac * T;
        assert!(
            quad.inwards_factor(d) <= lin.inwards_factor(d) + EPS,
            "frac={frac}"
        );
    }
}

#[test]
fn cubic_lags_behind_quadratic() {
    let cubic = Falloff::Cubic { threshold: T };
    let quad = Falloff::Quadratic { threshold: T };
    for frac in [0.1, 0.3, 0.5, 0.7, 0.9] {
        let d = frac * T;
        assert!(
            cubic.inwards_factor(d) <= quad.inwards_factor(d) + EPS,
            "frac={frac}"
        );
    }
}

#[test]
fn smoothstep_matches_closed_form() {
    let f = Falloff::SmoothStep { threshold: T };
    for frac in [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
        let expected = 3.0 * frac.powi(2) - 2.0 * frac.powi(3);
        assert!(
            (f.inwards_factor(frac * T) - expected).abs() < EPS,
            "frac={frac}"
        );
    }
}

#[test]
fn smoothstep_zero_derivative_at_boundaries() {
    let f = Falloff::SmoothStep { threshold: T };
    let h = T * 0.001;
    let slope_start = (f.inwards_factor(h) - f.inwards_factor(0.0)) / h;
    let slope_end = (f.inwards_factor(T) - f.inwards_factor(T - h)) / h;
    assert!(slope_start.abs() < 0.05, "slope at start: {slope_start}");
    assert!(slope_end.abs() < 0.05, "slope at end: {slope_end}");
}

#[test]
fn smootherstep_matches_closed_form() {
    let f = Falloff::SmootherStep { threshold: T };
    for frac in [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
        let expected = 6.0 * frac.powi(5) - 15.0 * frac.powi(4) + 10.0 * frac.powi(3);
        assert!(
            (f.inwards_factor(frac * T) - expected).abs() < EPS,
            "frac={frac}"
        );
    }
}

#[test]
fn smootherstep_hugs_boundaries_tighter_than_smoothstep() {
    let smoother = Falloff::SmootherStep { threshold: T };
    let smooth = Falloff::SmoothStep { threshold: T };
    for frac in [0.05, 0.1, 0.9, 0.95] {
        let d = frac * T;
        let dev_smoother = (smoother.inwards_factor(d) - frac).abs();
        let dev_smooth = (smooth.inwards_factor(d) - frac).abs();
        assert!(dev_smoother >= dev_smooth - EPS, "frac={frac}");
    }
}

#[test]
fn exponential_higher_exponent_rises_faster() {
    let low_k = Falloff::Exponential {
        threshold: T,
        exponent: 1.0,
    };
    let high_k = Falloff::Exponential {
        threshold: T,
        exponent: 5.0,
    };
    for frac in [0.1, 0.25, 0.5, 0.75] {
        let d = frac * T;
        assert!(
            high_k.inwards_factor(d) >= low_k.inwards_factor(d) - EPS,
            "frac={frac}: high={} low={}",
            high_k.inwards_factor(d),
            low_k.inwards_factor(d)
        );
    }
}

#[test]
fn inverse_square_convex_below_linear_midpoint() {
    let f = Falloff::InverseSquare { threshold: T };
    let mid = f.inwards_factor(T * 0.5);
    assert!(
        mid < 0.5,
        "midpoint should be below linear (0.5), got {mid}"
    );
}
