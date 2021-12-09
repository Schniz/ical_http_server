# `ical_http_server`

A server for my [Home Assistant](https://www.home-assistant.io/), installed as a [RESTful binary sensor](https://www.home-assistant.io/integrations/binary_sensor.rest/) to automate home appliances using a calendar.

The server is distributed as a very slim docker image that allows easy installation and integration in home servers.

## Why not using the default HA Calendar integration?

### Refresh rates

Unfortunately, the refresh rate of the different calendars are ~15min. This can create major delays and issues, especially when cancelling an automation in the last minute.

### Ad-hoc auth

Google allows using `ical` calendars that are protected behind a private URL. This is easier than integrating a calendar with the entire OAuth flow.

## API

### `POST /by_url`

Accepts JSON with the format of:

```json
{
  "urls": {
    "calendar_name_a": "https://url",
    "calendar_name_b": "https://url"
    // ...
  }
}
```

And returns the following payload:

```json
{
  "calendar_name_a": true,
  "calendar_name_b": false
  // ...
}
```

where `true` means "busy", and `false` means "free".

## Example usage with Home Assistant

```yaml
binary_sensor:
  - platform: rest
    name: boiler_calendar_busy
    scan_interval: 60
    resource: https://ical-sensor.my-home.server/by_url
    method: POST
    payload: |
      {"urls": {"boiler": "MY_ICAL_URL" }}
    value_template: "{{ value_json.boiler }}"
```

