// locale-match is a small library for matching user's preferred locales to available locales.  
// Copyright (C) © 2024  Petr Alexandrovich Sabanov
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! A small library for selecting the best match for user's preferred locales from available locales.
//!
//! The library consists of two modules:
//! * [`bcp47`] — for matching locales in the [BCP 47](https://www.ietf.org/rfc/bcp/bcp47.html) format.
//! * [`posix`] — for matching locales in the [POSIX](https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap08.html) format.
//!
//! Both modules provide the `best_matching_locale` function.
//! 
//! ## Examples
//! 
//! ### BCP 47
//!
//! ```
//! use locale_match::bcp47::best_matching_locale;
//!
//! let available_locales = ["en-US", "ru-BY"];
//! let user_locales = ["ru-RU", "ru", "en-US", "en"];
//!
//! let best_match = best_matching_locale(available_locales, user_locales);
//!
//! assert_eq!(best_match, Some("ru-BY"));
//! ```
//! 
//! ### POSIX
//! 
//! ```
//! use locale_match::posix::best_matching_locale;
//!
//! let available_locales = ["en_US.UTF-8", "ru_BY.UTF-8"];
//! let user_locales = ["ru_RU.UTF-8", "ru", "en_US.UTF-8", "en"];
//! 
//! let best_match = best_matching_locale(available_locales, user_locales);
//! 
//! assert_eq!(best_match, Some("ru_BY.UTF-8"));
//! ```

#[cfg(feature = "bcp47")]
pub mod bcp47;

#[cfg(feature = "posix")]
pub mod posix;