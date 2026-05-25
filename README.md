# ESP32-based epaper calendar

## Hardware:

- ESP32-s3 'Arduino' Nano
- WeAct Studio 2.9" epaper module (128x296, white-black-red)
- DS3231 RTC clock module

## Capabilities:

- SNTP time sync
- Fetching days off (Russia, Ukraine, Uzbekistan, Belarus, PRs for other countries / API providers are welcome)
- Two display styles: three months or current month and day
- Can run without network access if time sync and days off fetching are disabled

## Styles

### Current month + Today as big number (`calendar-style-bignum`)
<img width="1402" height="789" alt="bignum style" src="https://github.com/user-attachments/assets/cf7f5c94-fe9e-4ac6-bb59-17faeff497f3" />

### Current, previous and next months (`calendar-style-triplet`)
<img width="1402" height="789" alt="triplet style" src="https://github.com/user-attachments/assets/5a989c7d-0eb0-4381-9494-e0adef78207d" />

## Flashing:

First, you'll have to define some buildtime variables. You can specify them before build/flash command invocation (`WIFI_SSID="Your SSID here" cargo espflash ...`), or by putting them in a file `buildtime-vars`, one pair of variables on each line.

You will need https://crates.io/crates/cargo-espflash and the esp toolchain set up

`cargo espflash flash` for default configuration (`ntp`, `isdayoff`, `calendar-style-bignum`)

`cargo espflash flash --no-default-features --features <features>` for configuring the build, replace `<features>` with a comma-separated list of cargo features, WITHOUT SPACES

Add `-M` flag to see the bootup log

### User-facing cargo features:

- `isdayoff` - Enable days off fetching, currently via https://isdayoff.ru/
- `ntp` - enable SNTP time sync, if disabled, will try to use your local time while flashing
- `weather` - enable fetching weather forecast from open-meteo.com API
- `calendar-style-bignum` or `calendar-style-triplet` - enable ONE to select the display style
- `monthdate-packed` - experimental representation of MonthDate, potentially better memory utilization

### More customization

- isdayoff country can be changed in [`src/bin/async_main/isdayoff.rs`](src/bin/async_main/http_apis/isdayoff.rs) by changing the `TARGET_COUNTRY` constant
- Timezone can be changed in [`src/bin/async_main/time.rs`](src/bin/async_main/time.rs) by changing the `LOCAL_TZ` constant

## Pin mapping

| GPIO number | Board label | Destination            |
|-------------|-------------|------------------------|
| 11          | A4          | I2C SDA (DS3231)       |
| 12          | A5          | I2C SCK (DS3231)       |
| 5           | D2          | Display CS             |
| 4           | A3          | Display BUSY           |
| 10          | D7          | Display RST / RES      |
| 17          | D8          | Display D/C            |
| 18          | D9          | Display SCL / SPI SCK  |
| 21          | D10         | Display SDA / SPI MOSI |

Power is 3.3v from the dev board for both display and clock.
