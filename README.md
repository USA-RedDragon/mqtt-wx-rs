# mqtt-wx

[![codecov](https://codecov.io/gh/USA-RedDragon/mqtt-wx-rs/graph/badge.svg?token=8dBAphXo0c)](https://codecov.io/gh/USA-RedDragon/mqtt-wx-rs) [![License](https://badgen.net/github/license/USA-RedDragon/mqtt-wx-rs)](https://github.com/USA-RedDragon/mqtt-wx-rs/blob/main/LICENSE) [![GitHub contributors](https://badgen.net/github/contributors/USA-RedDragon/mqtt-wx-rs)](https://github.com/USA-RedDragon/mqtt-wx-rs/graphs/contributors/)


This is a little translation layer between rtl_433 and Home Assistant + WeeWX for a Cotech 36-7959 Weatherstation or other compatible models. It takes in multiple MQTT topics (weather station, indoor module, lightning, light, pressure, particle sensor, CO2) and coalesces them into a single output topic with computed meteorological values.

Note: This is a form-fit translation layer between various weather-related sensors I personally have. I will not support use of this tool, but do provide it as an example to others who might want to do something similar.

For WeeWX, this uses <https://github.com/USA-RedDragon/weewxMQTT> to read weather data from MQTT. An example `weewx.conf` entry can be found in `examples/weewx.conf`.

A Home Assistant example config can be found in `examples/home_assistant.yaml`.

This project is a single-purpose project and does not accept bug reports or most PRs.

## Configuration

Configuration is loaded from multiple sources with the following precedence (highest wins):

1. **Defaults** (built-in)
2. **Config file** (`config.yaml` in the current directory, or `/etc/mqtt-wx/config.yaml`)
3. **Environment variables** (prefixed with `MQTT_WX__`, e.g. `MQTT_WX__MQTT__HOST`)
4. **CLI flags** (e.g. `--mqtt.host`)

### Config file

Create a `config.yaml`:

```yaml
mqtt:
  host: "192.168.1.100"
  port: 1883
  username: "user"
  password: "pass"
input-topic:
  weather: "rtl_433/weather"
  indoor: "rtl_433/indoor"
  lightning: "rtl_433/lightning"
  light: "rtl_433/light"
  pressure: "rtl_433/pressure"
  particle-sensor: "rtl_433/particle"
  co2: "rtl_433/co2"
output-topic: "mqtt-wx/output"
sensor-height-m: 2.7432
elevation-m: 363.2
```

### Environment variables

All config options can be set via environment variables with the prefix `MQTT_WX` and separator `__`:

| Environment Variable | Description |
| --- | --- |
| `MQTT_WX__MQTT__HOST` | MQTT broker hostname (default: `localhost`) |
| `MQTT_WX__MQTT__PORT` | MQTT broker port (default: `1883`) |
| `MQTT_WX__MQTT__USERNAME` | MQTT username |
| `MQTT_WX__MQTT__PASSWORD` | MQTT password |
| `MQTT_WX__INPUT_TOPIC__WEATHER` | Input topic for weather station data |
| `MQTT_WX__INPUT_TOPIC__INDOOR` | Input topic for indoor sensor data |
| `MQTT_WX__INPUT_TOPIC__LIGHTNING` | Input topic for lightning data |
| `MQTT_WX__INPUT_TOPIC__LIGHT` | Input topic for light data |
| `MQTT_WX__INPUT_TOPIC__PRESSURE` | Input topic for pressure data |
| `MQTT_WX__INPUT_TOPIC__PARTICLE_SENSOR` | Input topic for particle sensor data |
| `MQTT_WX__INPUT_TOPIC__CO2` | Input topic for CO2 sensor data |
| `MQTT_WX__OUTPUT_TOPIC` | Output topic for processed data |
| `MQTT_WX__SENSOR_HEIGHT_M` | Sensor height above ground in meters (default: `2.7432`) |
| `MQTT_WX__ELEVATION_M` | Field elevation in meters above sea level (default: `363.2`) |

### CLI flags

```bash
mqtt-wx --mqtt.host 192.168.1.100 --mqtt.port 1883 --mqtt.username user --mqtt.password pass
```

Run `mqtt-wx --help` for the full list of options.

## Computed values

The following meteorological values are computed from the raw sensor data:

- **Dew point** (Arden Buck equation)
- **Heat index** (NOAA formula with Rothfusz regression)
- **Wind chill** (NOAA formula, applicable when temp < 50°F and wind >= 3 mph)
- **Frost point**
- **Cloud base** (LCL formula)
- **24-hour rain accumulation** (sliding window)

## Sanity checking

Output data is validated against reasonable ranges and delta checks before publishing. Readings outside expected bounds (e.g. temperature outside -50°F to 150°F, or a jump of more than 30°F between readings) are rejected.
