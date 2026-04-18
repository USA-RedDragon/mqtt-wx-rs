use crate::types::OutputData;

/// Validates that output data values are within expected physical ranges
/// and that changes from previous readings aren't unreasonably large.
pub fn sanity_check(data: &OutputData, prev_data: &OutputData) -> bool {
    if let Some(t) = data.out_temp {
        if !(-50.0..=150.0).contains(&t) {
            return false;
        }
        let prev = prev_data.out_temp.unwrap_or(t);
        if (t - prev).abs() > 30.0 {
            return false;
        }
    }

    if let Some(h) = data.out_humidity {
        if !(0.0..=100.0).contains(&h) {
            return false;
        }
        let prev = prev_data.out_humidity.unwrap_or(h);
        if (h - prev).abs() > 30.0 {
            return false;
        }
    }

    if let Some(d) = data.wind_dir
        && (!(0.0..=360.0).contains(&d)) {
            return false;
        }

    if let Some(s) = data.wind_speed
        && (!(0.0..=200.0).contains(&s)) {
            return false;
        }

    if let Some(g) = data.wind_gust
        && (!(0.0..=200.0).contains(&g)) {
            return false;
        }

    if let Some(hi) = data.heatindex
        && (!(-50.0..=150.0).contains(&hi)) {
            return false;
        }

    if let Some(wc) = data.windchill
        && (!(-50.0..=150.0).contains(&wc)) {
            return false;
        }

    if let Some(dp) = data.dewpoint
        && (!(-50.0..=150.0).contains(&dp)) {
            return false;
        }

    if let Some(fp) = data.frostpoint
        && (!(-50.0..=150.0).contains(&fp)) {
            return false;
        }

    if let Some(t) = data.in_temp {
        if !(-50.0..=150.0).contains(&t) {
            return false;
        }
        let prev = prev_data.in_temp.unwrap_or(t);
        if (t - prev).abs() > 30.0 {
            return false;
        }
    }

    if let Some(h) = data.in_humidity {
        if !(0.0..=100.0).contains(&h) {
            return false;
        }
        let prev = prev_data.in_humidity.unwrap_or(h);
        if (h - prev).abs() > 30.0 {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_data() -> OutputData {
        OutputData {
            out_temp: Some(22.0),
            out_humidity: Some(50.0),
            wind_dir: Some(180.0),
            wind_speed: Some(5.0),
            wind_gust: Some(10.0),
            heatindex: Some(25.0),
            dewpoint: Some(12.0),
            frostpoint: Some(10.0),
            in_temp: Some(21.0),
            in_humidity: Some(45.0),
            ..Default::default()
        }
    }

    #[test]
    fn test_valid_data_passes() {
        let data = default_data();
        let prev = default_data();
        assert!(sanity_check(&data, &prev));
    }

    #[test]
    fn test_empty_data_passes() {
        let data = OutputData::default();
        let prev = OutputData::default();
        assert!(sanity_check(&data, &prev));
    }

    #[test]
    fn test_out_temp_too_low() {
        let mut data = default_data();
        data.out_temp = Some(-60.0);
        assert!(!sanity_check(&data, &default_data()));
    }

    #[test]
    fn test_out_temp_too_high() {
        let mut data = default_data();
        data.out_temp = Some(160.0);
        assert!(!sanity_check(&data, &default_data()));
    }

    #[test]
    fn test_out_temp_large_delta() {
        let mut data = default_data();
        data.out_temp = Some(55.0); // 33 degree jump from 22
        assert!(!sanity_check(&data, &default_data()));
    }

    #[test]
    fn test_out_humidity_too_low() {
        let mut data = default_data();
        data.out_humidity = Some(-1.0);
        assert!(!sanity_check(&data, &default_data()));
    }

    #[test]
    fn test_out_humidity_too_high() {
        let mut data = default_data();
        data.out_humidity = Some(101.0);
        assert!(!sanity_check(&data, &default_data()));
    }

    #[test]
    fn test_out_humidity_large_delta() {
        let mut data = default_data();
        data.out_humidity = Some(85.0); // 35 point jump from 50
        assert!(!sanity_check(&data, &default_data()));
    }

    #[test]
    fn test_wind_dir_too_low() {
        let mut data = default_data();
        data.wind_dir = Some(-1.0);
        assert!(!sanity_check(&data, &default_data()));
    }

    #[test]
    fn test_wind_dir_too_high() {
        let mut data = default_data();
        data.wind_dir = Some(361.0);
        assert!(!sanity_check(&data, &default_data()));
    }

    #[test]
    fn test_wind_speed_negative() {
        let mut data = default_data();
        data.wind_speed = Some(-1.0);
        assert!(!sanity_check(&data, &default_data()));
    }

    #[test]
    fn test_wind_speed_too_high() {
        let mut data = default_data();
        data.wind_speed = Some(201.0);
        assert!(!sanity_check(&data, &default_data()));
    }

    #[test]
    fn test_wind_gust_negative() {
        let mut data = default_data();
        data.wind_gust = Some(-5.0);
        assert!(!sanity_check(&data, &default_data()));
    }

    #[test]
    fn test_heatindex_out_of_range() {
        let mut data = default_data();
        data.heatindex = Some(160.0);
        assert!(!sanity_check(&data, &default_data()));
    }

    #[test]
    fn test_windchill_out_of_range() {
        let mut data = default_data();
        data.windchill = Some(-60.0);
        assert!(!sanity_check(&data, &default_data()));
    }

    #[test]
    fn test_dewpoint_out_of_range() {
        let mut data = default_data();
        data.dewpoint = Some(200.0);
        assert!(!sanity_check(&data, &default_data()));
    }

    #[test]
    fn test_frostpoint_out_of_range() {
        let mut data = default_data();
        data.frostpoint = Some(-55.0);
        assert!(!sanity_check(&data, &default_data()));
    }

    #[test]
    fn test_in_temp_large_delta() {
        let mut data = default_data();
        data.in_temp = Some(55.0); // 34 degree jump from 21
        assert!(!sanity_check(&data, &default_data()));
    }

    #[test]
    fn test_in_humidity_out_of_range() {
        let mut data = default_data();
        data.in_humidity = Some(110.0);
        assert!(!sanity_check(&data, &default_data()));
    }

    #[test]
    fn test_no_prev_data_uses_current_as_baseline() {
        let data = default_data();
        let prev = OutputData::default();
        assert!(sanity_check(&data, &prev));
    }

    #[test]
    fn test_windchill_none_passes() {
        let mut data = default_data();
        data.windchill = None;
        assert!(sanity_check(&data, &default_data()));
    }
}
