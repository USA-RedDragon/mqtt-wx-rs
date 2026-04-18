pub fn f_to_c(temp_f: f64) -> f64 {
    (temp_f - 32.0) * 5.0 / 9.0
}

pub fn c_to_k(temp_c: f64) -> f64 {
    temp_c + 273.15
}

pub fn k_to_f(temp_k: f64) -> f64 {
    temp_k * 9.0 / 5.0 - 459.67
}

pub fn mps_to_mph(wind_mps: f64) -> f64 {
    wind_mps * 2.23694
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f_to_c_freezing() {
        assert!((f_to_c(32.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_f_to_c_boiling() {
        assert!((f_to_c(212.0) - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_f_to_c_body_temp() {
        assert!((f_to_c(98.6) - 37.0).abs() < 0.01);
    }

    #[test]
    fn test_f_to_c_negative() {
        assert!((f_to_c(-40.0) - -40.0).abs() < 1e-10);
    }

    #[test]
    fn test_c_to_k_freezing() {
        assert!((c_to_k(0.0) - 273.15).abs() < 1e-10);
    }

    #[test]
    fn test_c_to_k_absolute_zero() {
        assert!((c_to_k(-273.15) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_k_to_f_absolute_zero() {
        assert!((k_to_f(0.0) - -459.67).abs() < 1e-10);
    }

    #[test]
    fn test_k_to_f_boiling() {
        assert!((k_to_f(373.15) - 212.0).abs() < 0.01);
    }

    #[test]
    fn test_round_trip_f_c_f() {
        let original = 72.5;
        let result = f_to_c(original) * 9.0 / 5.0 + 32.0;
        assert!((result - original).abs() < 1e-10);
    }

    #[test]
    fn test_round_trip_c_k_c() {
        let original = 25.0;
        let result = c_to_k(original) - 273.15;
        assert!((result - original).abs() < 1e-10);
    }

    #[test]
    fn test_mps_to_mph() {
        assert!((mps_to_mph(1.0) - 2.23694).abs() < 1e-5);
    }

    #[test]
    fn test_mps_to_mph_zero() {
        assert!((mps_to_mph(0.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_mps_to_mph_ten() {
        assert!((mps_to_mph(10.0) - 22.3694).abs() < 1e-4);
    }
}
