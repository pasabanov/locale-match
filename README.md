# locale-match

[![crates.io version](https://img.shields.io/crates/v/locale-match?style=for-the-badge&logo=rust)](https://crates.io/crates/locale-match)
[![docs.rs documentation](https://img.shields.io/docsrs/locale-match/latest?style=for-the-badge&logo=docs.rs&color=2DA44E)](https://docs.rs/locale-match/latest/locale_match)
[![GitHub latest release](https://img.shields.io/github/v/release/pasabanov/locale-match?style=for-the-badge&logo=github&color=8250DF)](https://github.com/pasabanov/locale-match/releases/latest)

A small library for selecting the best match for user's preferred locales from available locales.

## Examples

### BCP 47

```rust
use locale_match::bcp47::best_matching_locale;


let available_locales = ["en-US", "en-GB", "ru-UA", "fr-FR", "it"];
let user_locales = ["ru-RU", "ru", "en-US", "en"];

let best_match = best_matching_locale(available_locales, user_locales);

// "ru-UA" is the best match for the highest-priority user locale "ru-RU"
assert_eq!(best_match, Some("ru-UA"));


let available_locales = ["en", "pt-BR", "pt-PT", "es"];
let user_locales = ["pt", "en"];

let best_match = best_matching_locale(available_locales, user_locales);

// "pt-BR" is the first best match for the highest-priority user locale "pt"
assert_eq!(best_match, Some("pt-BR"));


let available_locales = ["zh", "zh-cmn", "zh-cmn-Hans"];
let user_locales = ["zh-Hans"];

let best_match = best_matching_locale(available_locales, user_locales);

// Empty extended language subtag in "zh-Hans" matches any extended language, e.g. "cmn"
assert_eq!(best_match, Some("zh-cmn-Hans"));
```

### POSIX

```rust
use locale_match::posix::best_matching_locale;


let available_locales = ["en_US", "en_GB", "ru_UA", "fr_FR", "it"];
let user_locales = ["ru_RU", "ru", "en_US", "en"];

let best_match = best_matching_locale(available_locales, user_locales);

// "ru_UA" is the best match for the highest-priority user locale "ru_RU"
assert_eq!(best_match, Some("ru_UA"));


let available_locales = ["en", "pt_BR", "pt_PT", "es"];
let user_locales = ["pt", "en"];

let best_match = best_matching_locale(available_locales, user_locales);

// "pt_BR" is the first best match for the highest-priority user locale "pt"
assert_eq!(best_match, Some("pt_BR"));


let available_locales = ["fr", "fr_FR", "fr_CA.UTF-8"];
let user_locales = ["fr.UTF-8"];

let best_match = best_matching_locale(available_locales, user_locales);

// Empty territory in "fr.UTF-8" matches any territory, e.g. "CA"
assert_eq!(best_match, Some("fr_CA.UTF-8"));
```

## License

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Lesser General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Lesser General Public License for more details.

You should have received a copy of the GNU Lesser General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.

## Copyright

© 2024 Petr Alexandrovich Sabanov

## Metrics

![repo size](https://img.shields.io/github/repo-size/pasabanov/locale-match?color=8250DF)
![crate size](https://img.shields.io/crates/size/locale-match?label=crate%20size&color=orange)