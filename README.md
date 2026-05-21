# ESP32-based epaper calendar

## Hardware:

- ESP32-s3 'Arduino' Nano
- WeAct Studio 2.9" epaper module (128x296, white-black-red)
- DS3231 RTC clock module

## Flashing:

You will need https://crates.io/crates/cargo-espflash and the esp toolchain set up

`cargo espflash flash` for default configuration (`ntp`, `isdayoff`, `calendar-style-bignum`)

`cargo espflash flash --no-default-features --features <features>` for configuring the build, replace `<features>` with a comma-separated list of cargo features, WITHOUT SPACES

Add `-M` flag to see the bootup log

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

### User-facing cargo features:

- `isdayoff` - Enable days off fetching, currently via https://isdayoff.ru/
- `ntp` - enable SNTP time sync
- `calendar-style-bignum` or `calendar-style-triplet` - enable ONE to select the display style
- `monthdate-packed` - experimental representation of MonthDate, potentially better memory utilization
