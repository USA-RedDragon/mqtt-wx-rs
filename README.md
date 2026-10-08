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

Copy [config.example.yaml](config.example.yaml) to `config.yaml` to get started. Every option:

<!-- configulator:begin -->

| Key                           | Type    | Default           | Environment                             | Flag                            | Description                               |
|-------------------------------|---------|-------------------|-----------------------------------------|---------------------------------|-------------------------------------------|
| `mqtt.host`                   | string  | `localhost`       | `MQTT_WX__MQTT__HOST`                   | `--mqtt.host`                   | MQTT broker hostname                      |
| `mqtt.port`                   | integer | `1883`            | `MQTT_WX__MQTT__PORT`                   | `--mqtt.port`                   | MQTT broker port                          |
| `mqtt.username`               | string  |                   | `MQTT_WX__MQTT__USERNAME`               | `--mqtt.username`               | MQTT username                             |
| `mqtt.password`               | string  |                   | `MQTT_WX__MQTT__PASSWORD`               | `--mqtt.password`               | MQTT password                             |
| `input-topic.weather`         | string  | `weather`         | `MQTT_WX__INPUT_TOPIC__WEATHER`         | `--input-topic.weather`         | Input topic for weather station data      |
| `input-topic.indoor`          | string  | `indoor`          | `MQTT_WX__INPUT_TOPIC__INDOOR`          | `--input-topic.indoor`          | Input topic for indoor sensor data        |
| `input-topic.lightning`       | string  | `lightning`       | `MQTT_WX__INPUT_TOPIC__LIGHTNING`       | `--input-topic.lightning`       | Input topic for lightning data            |
| `input-topic.light`           | string  | `light`           | `MQTT_WX__INPUT_TOPIC__LIGHT`           | `--input-topic.light`           | Input topic for light data                |
| `input-topic.pressure`        | string  | `pressure`        | `MQTT_WX__INPUT_TOPIC__PRESSURE`        | `--input-topic.pressure`        | Input topic for pressure data             |
| `input-topic.particle-sensor` | string  | `particle_sensor` | `MQTT_WX__INPUT_TOPIC__PARTICLE_SENSOR` | `--input-topic.particle-sensor` | Input topic for particle sensor data      |
| `input-topic.co2`             | string  | `co2`             | `MQTT_WX__INPUT_TOPIC__CO2`             | `--input-topic.co2`             | Input topic for CO2 sensor data           |
| `output-topic`                | string  | `processed`       | `MQTT_WX__OUTPUT_TOPIC`                 | `--output-topic`                | Output topic for processed data           |
| `sensor-height-m`             | number  | `2.7432`          | `MQTT_WX__SENSOR_HEIGHT_M`              | `--sensor-height-m`             | Sensor height above ground in meters      |
| `elevation-m`                 | number  | `363.2`           | `MQTT_WX__ELEVATION_M`                  | `--elevation-m`                 | Field elevation in meters above sea level |

<!-- configulator:end -->

Run `mqtt-wx --help` for the flags.

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
