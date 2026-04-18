use crate::units::k_to_f;

/// Calculates the dew point using Arden Buck's method.
///
/// # Arguments
/// * `temp_f` - Dry-bulb temperature in degrees Fahrenheit.
/// * `relative_humidity` - Relative humidity as a percentage (e.g. 50 for 50%).
///
/// # Returns
/// Dew point temperature in degrees Fahrenheit.
pub fn dew_point(temp_f: f64, relative_humidity: f64) -> f64 {
    let t = (temp_f - 32.0) * 5.0 / 9.0; // Convert to Celsius
    let rh = relative_humidity;

    let (a, b, c) = if t >= 0.0 {
        (6.1121, 17.368, 238.88)
    } else {
        (6.1121, 17.966, 247.15)
    };

    let p_vapor_saturation = a * ((b * t) / (c + t)).exp();
    let p_vapor_actual = (rh / 100.0) * p_vapor_saturation;

    let dew_point_c = (c * (p_vapor_actual / a).ln()) / (b - (p_vapor_actual / a).ln());

    // Convert back to Fahrenheit
    dew_point_c * 9.0 / 5.0 + 32.0
}

// Heat Index coefficients
// <https://www.wpc.ncep.noaa.gov/html/heatindex_equation.shtml>
const C1: f64 = -42.379;
const C2: f64 = 2.04901523;
const C3: f64 = 10.14333127;
const C4: f64 = 0.22475541;
const C5: f64 = 0.00683783;
const C6: f64 = 0.05481717;
const C7: f64 = 0.00122874;
const C8: f64 = 0.00085282;
const C9: f64 = 0.00000199;

/// Calculates the heat index using the NOAA WPC method.
///
/// # Arguments
/// * `temp_f` - Temperature in degrees Fahrenheit.
/// * `relative_humidity` - Relative humidity as a percentage.
///
/// # Returns
/// Heat index in degrees Fahrenheit.
pub fn heat_index(temp_f: f64, relative_humidity: f64) -> f64 {
    let t = temp_f;
    let rh = relative_humidity;

    let hi = C1 + C2 * t + C3 * rh - C4 * t * rh - C5 * t * t - C6 * rh * rh
        + C7 * t * t * rh
        + C8 * t * rh * rh
        - C9 * t * t * rh * rh;

    if rh < 13.0 && (80.0..=112.0).contains(&t) {
        hi - ((13.0 - rh) / 4.0) * ((17.0 - (t - 95.0).abs()) / 17.0).sqrt()
    } else if rh > 85.0 && (80.0..=87.0).contains(&t) {
        hi + ((rh - 85.0) / 10.0) * ((87.0 - t) / 5.0)
    } else {
        0.5 * (t + 61.0 + ((t - 68.0) * 1.2) + (rh * 0.094))
    }
}

/// Calculates wind chill using the NOAA formula.
///
/// # Arguments
/// * `temp_f` - Temperature in degrees Fahrenheit.
/// * `wind_speed_mph` - Wind speed in miles per hour.
///
/// # Returns
/// A tuple of (wind_chill_f, applicable) where `applicable` indicates whether wind chill
/// conditions are met (temp < 50°F and wind >= 3 mph).
pub fn wind_chill(temp_f: f64, wind_speed_mph: f64) -> (f64, bool) {
    if wind_speed_mph < 3.0 || temp_f >= 50.0 {
        return (0.0, false);
    }
    let wc = 35.74 + 0.6215 * temp_f - 35.75 * wind_speed_mph.powf(0.16)
        + 0.4275 * temp_f * wind_speed_mph.powf(0.16);
    (wc, true)
}

/// Calculates the frost point.
///
/// # Arguments
/// * `temp_k` - Temperature in Kelvin.
/// * `dewpoint_k` - Dew point temperature in Kelvin.
///
/// # Returns
/// Frost point temperature in degrees Fahrenheit.
pub fn frost_point(temp_k: f64, dewpoint_k: f64) -> f64 {
    k_to_f(
        dewpoint_k - temp_k
            + 2671.02 / ((2954.61 / temp_k) + 2.193665 * temp_k.ln() - 13.3448),
    )
}

/// Calculates the cloud base height using the LCL formula.
///
/// # Arguments
/// * `temp_c` - Temperature in Celsius.
/// * `dewpoint_c` - Dew point in Celsius.
///
/// # Returns
/// Cloud base height in meters (above the sensor).
pub fn cloudbase(temp_c: f64, dewpoint_c: f64) -> f64 {
    (temp_c - dewpoint_c) / 2.4 * 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dew_point_typical() {
        // At 72°F and 50% RH, dew point should be around 52°F
        let dp = dew_point(72.0, 50.0);
        assert!((dp - 52.6).abs() < 1.0, "dew point was {dp}");
    }

    #[test]
    fn test_dew_point_high_humidity() {
        // At 100% RH, dew point should equal the temperature
        let dp = dew_point(72.0, 100.0);
        assert!((dp - 72.0).abs() < 0.5, "dew point was {dp}");
    }

    #[test]
    fn test_dew_point_low_humidity() {
        // At very low humidity, dew point should be well below air temp
        let dp = dew_point(90.0, 10.0);
        assert!(dp < 40.0, "dew point was {dp}");
    }

    #[test]
    fn test_dew_point_freezing() {
        // Sub-freezing temperature
        let dp = dew_point(20.0, 50.0);
        assert!(dp < 20.0, "dew point was {dp}");
    }

    #[test]
    fn test_heat_index_hot_humid() {
        // At 95°F and 50% RH, heat index is typically > 95
        let hi = heat_index(95.0, 50.0);
        assert!(hi > 95.0, "heat index was {hi}");
    }

    #[test]
    fn test_heat_index_moderate() {
        // At 72°F and 50% RH, simple formula applies
        let hi = heat_index(72.0, 50.0);
        assert!((hi - 70.0).abs() < 5.0, "heat index was {hi}");
    }

    #[test]
    fn test_heat_index_low_rh_adjustment() {
        // Low RH (< 13%) with temp between 80-112 triggers adjustment
        let hi = heat_index(95.0, 10.0);
        assert!(hi > 80.0, "heat index was {hi}");
    }

    #[test]
    fn test_heat_index_high_rh_adjustment() {
        // High RH (> 85%) with temp between 80-87 triggers adjustment
        let hi = heat_index(85.0, 90.0);
        assert!(hi > 85.0, "heat index was {hi}");
    }

    #[test]
    fn test_wind_chill_cold_windy() {
        // At 30°F and 10 mph wind, wind chill should be below 30
        let (wc, applicable) = wind_chill(30.0, 10.0);
        assert!(applicable);
        assert!(wc < 30.0, "wind chill was {wc}");
    }

    #[test]
    fn test_wind_chill_not_applicable_warm() {
        // Wind chill doesn't apply above 50°F
        let (_, applicable) = wind_chill(55.0, 10.0);
        assert!(!applicable);
    }

    #[test]
    fn test_wind_chill_not_applicable_calm() {
        // Wind chill doesn't apply below 3 mph
        let (_, applicable) = wind_chill(30.0, 2.0);
        assert!(!applicable);
    }

    #[test]
    fn test_wind_chill_reference_value() {
        // NOAA reference: 0°F at 15mph -> wind chill around -19°F
        let (wc, applicable) = wind_chill(0.0, 15.0);
        assert!(applicable);
        assert!((wc - -19.0).abs() < 2.0, "wind chill was {wc}");
    }

    #[test]
    fn test_frost_point_above_freezing() {
        // Frost point should be near but slightly below dew point for temps above freezing
        let temp_k = 293.15; // 20°C
        let dewpoint_k = 283.15; // 10°C
        let fp = frost_point(temp_k, dewpoint_k);
        // Frost point should be cold but reasonable
        assert!(fp < 60.0, "frost point was {fp}");
    }

    #[test]
    fn test_cloudbase_typical() {
        // With 5°C spread, cloudbase ~2083m above sensor
        let cb = cloudbase(20.0, 15.0);
        assert!((cb - 2083.3).abs() < 1.0, "cloudbase was {cb}");
    }

    #[test]
    fn test_cloudbase_zero_spread() {
        // If temp equals dewpoint, cloudbase should be 0 (fog)
        let cb = cloudbase(15.0, 15.0);
        assert!((cb - 0.0).abs() < 0.01, "cloudbase was {cb}");
    }

    #[test]
    fn test_cloudbase_large_spread() {
        // Large temp-dewpoint spread -> high cloudbase
        let cb = cloudbase(30.0, 10.0);
        assert!((cb - 8333.3).abs() < 1.0, "cloudbase was {cb}");
    }
}
