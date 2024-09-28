//! locale-match is a small library for matching user's preferred locales to available locales.  
//! Copyright (C) © 2024  Petr Alexandrovich Sabanov
//!
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU Lesser General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU Lesser General Public License for more details.
//!
//! You should have received a copy of the GNU Lesser General Public License
//! along with this program.  If not, see <https://www.gnu.org/licenses/>.

use language_tags::LanguageTag;

/// Finds the best matching locale from a list of available locales based on a list of user locales.
/// The function ignores any locales that are not valid BCP 47 locales.
///
/// The function compares user locales to available locales to find the best match.
/// For each user locale, it iterates through the available locales and, for those with a matching
/// primary language, calculates a score based on how closely each available locale matches the user
/// locale.
/// The score calculation gives higher priority to matching more significant parts of the locale
/// (i.e., earlier segments in the locale string).
/// If a subtag is empty, it is considered to match equally well with any subtag from the same
/// category.
///
/// If multiple available locales have the same score, the function selects the one that appears
/// earlier in the list of available locales.
/// If no available locale matches the primary language of a user locale, the function moves to the
/// next user locale in the list.
/// If no matches are found for any user locale, the function returns [`None`].
///
/// Malformed locales are ignored.
///
/// # Arguments
///
/// * `available_locales` - An iterator over locale strings representing the available locales.
///   These locales should be ordered by priority, meaning that a locale appearing earlier in this
///   list is considered more preferable for the program.
/// * `user_locales` - An iterator over locale strings representing the user locales to match
///   against. These locales should also be ordered by priority, meaning that a locale appearing
///   earlier in this list is considered more desirable for the user.
///
/// # Returns
///
/// Returns an [`Option<String>`] containing the string representation of the best matching locale.
/// If multiple available locales match the same user locale with equal score, the one that appears
/// earlier in the list of available locales is chosen.
/// If no match is found, [`None`] is returned.
///
/// The returned locale is guaranteed to EXACTLY match one of the available locales.
/// For example, `best_matching_locale(["EN"], ["en"])` will return `Some("EN")`.
///
/// # Examples
///
/// ```
/// use locale_match::bcp47::best_matching_locale;
///
///
/// let available_locales = ["en-US", "en-GB", "ru-UA", "fr-FR", "it"];
/// let user_locales = ["ru-RU", "ru", "en-US", "en"];
///
/// let best_match = best_matching_locale(available_locales, user_locales);
///
/// // "ru-UA" is the best match for the highest-priority user locale "ru-RU"
/// assert_eq!(best_match, Some("ru-UA"));
///
///
/// let available_locales = ["en", "pt-BR", "pt-PT", "es"];
/// let user_locales = ["pt", "en"];
///
/// let best_match = best_matching_locale(available_locales, user_locales);
///
/// // "pt-BR" is the first best match for the highest-priority user locale "pt"
/// assert_eq!(best_match, Some("pt-BR"));
///
///
/// let available_locales = ["zh", "zh-cmn", "zh-cmn-Hans"];
/// let user_locales = ["zh-Hans"];
///
/// let best_match = best_matching_locale(available_locales, user_locales);
///
/// // Empty extended language subtag in "zh-Hans" matches any extended language, e.g. "cmn"
/// assert_eq!(best_match, Some("zh-cmn-Hans"));
/// ```
pub fn best_matching_locale<T1, T2>(available_locales: impl IntoIterator<Item = T1>, user_locales: impl IntoIterator<Item = T2>) -> Option<T1>
where
	T1: AsRef<str>,
	T2: AsRef<str>
{
	let available_tags = available_locales.into_iter()
		.filter_map(|l| LanguageTag::parse(l.as_ref()).ok().map(|tag| (l, tag)))
		.collect::<Vec<(T1, LanguageTag)>>();

	user_locales.into_iter()
		.filter_map(|locale| LanguageTag::parse(locale.as_ref()).ok())
		.find_map(|user_tag|
			available_tags.iter()
				.enumerate()
				.rev() // For max_by_key to return the first tag with max score
				.filter(|(_, (_, aval_tag))| aval_tag.primary_language() == user_tag.primary_language())
				.max_by_key(|(_, (_, aval_tag))| {
					let mut score = 0;
					for (aval, user, weight) in [
						(aval_tag.extended_language(), user_tag.extended_language(), 32),
						(aval_tag.script(),            user_tag.script(),            16),
						(aval_tag.region(),            user_tag.region(),             8),
						(aval_tag.variant(),           user_tag.variant(),            4),
						// TODO: Implement separate comparison for each extension
						(aval_tag.extension(),         user_tag.extension(),          2),
						(aval_tag.private_use(),       user_tag.private_use(),        1),
					] {
						match (aval, user) {
							(Some(a), Some(u)) if a == u => score += weight,
							_ => {} // Ignore if both are None
						}
					}
					score
				})
				.map(|(i, _)| i)
		)
		.map(|i| available_tags.into_iter().nth(i).unwrap().0)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_best_matching_locale() {

		fn case<T1, T2>(available_locales: impl IntoIterator<Item = T1>, user_locales: impl IntoIterator<Item = T2>, expected: Option<T1>)
		where
			T1: AsRef<str> + PartialEq + std::fmt::Debug,
			T2: AsRef<str>
		{
			assert_eq!(best_matching_locale(available_locales, user_locales), expected);
		}

		// One best match
		case(["en-US", "ru-RU"], ["ru", "en"], Some("ru-RU"));
		case(["en-US", "ru-RU"], ["en", "ru"], Some("en-US"));
		case(["en-US", "en-GB", "ru-UA", "fr-FR", "it"], ["ru-RU", "ru", "en-US", "en"], Some("ru-UA"));
		case(["ru-RU", "sq-AL", "eu-ES"], ["en-US", "en", "sq-XK", "sq"], Some("sq-AL"));
		case(["lv-LV", "ru-RU", "lt-LT", "mn-MN", "ku-TR"], ["fr", "fr-FR", "ml", "si", "id", "ku-IQ"], Some("ku-TR"));
		case(["st-LS", "sn-ZW", "en-US"], ["zu-ZA", "st-ZA", "en"], Some("st-LS"));

		// Multiple best matches
		case(["en-US", "en-GB", "ru-UA", "fr-FR", "it"], ["en-US", "en", "ru-RU", "ru"], Some("en-US"));
		case(["en", "pt-BR", "pt-PT", "es"], ["pt", "en"], Some("pt-BR"));
		case(["ku-TR", "ku-IQ", "ku-IR"], ["ku", "en"], Some("ku-TR"));
		case(["en-US", "ru-RU", "mn-CN", "sn-ZW", "en", "ru", "mn-MN", "sn"], ["mn", "ru", "en", "sn"], Some("mn-CN"));

		// Identical
		case(["en"], ["en"], Some("en"));
		case(["en-US"], ["en-US"], Some("en-US"));
		case(["en-US", "ru-RU"], ["en-US", "ru-RU"], Some("en-US"));
		case(["st-LS", "sn-ZW", "en-US"], ["st-LS", "sn-ZW", "en-US"], Some("st-LS"));
		case(["ku-TR", "ku-IQ", "ku-IR"], ["ku-TR", "ku-IQ", "ku-IR"], Some("ku-TR"));
		case(["lv-LV", "ru-RU", "lt-LT", "mn-MN", "ku-TR"], ["lv-LV", "ru-RU", "lt-LT", "mn-MN", "ku-TR"], Some("lv-LV"));

		// One available locale
		case(["kk"], ["en", "en-US", "fr-FR", "fr", "it", "pt", "ru-RU", "es-ES", "kk-KZ"], Some("kk"));

		// One user locale
		case(["en", "en-US", "fr-FR", "fr", "it", "pt", "ru-RU", "es-ES", "kk-KZ", "pt"], ["pt-PT"], Some("pt"));

		// Not found
		case(["en", "en-US", "fr-FR", "fr", "it", "pt", "es-ES", "kk-KZ", "pt"], ["ru"], None);
		case(["en", "en-US", "fr-FR", "fr", "pt"], ["id"], None);
		case(["ru", "be", "uk", "kk"], ["en"], None);

		// Empty available locales
		case(&[] as &[&str], &["en", "fr", "it", "pt"], None);

		// Empty user locales
		case(["en", "fr", "it", "pt"], &[] as &[&str], None);

		// Both lists empty
		case(&[] as &[&str], &[] as &[&str], None);

		// More subtags
		case(["zh", "zh-cmn", "zh-cmn-Hans"], ["zh-cmn-SG"], Some("zh-cmn"));
		case(["zh", "zh-cmn", "zh-cmn-Hans", "zh-cmn-Hans-SG"], ["zh-cmn-SG"], Some("zh-cmn-Hans-SG"));
		case(["zh", "zh-cmn", "zh-cmn-Hans-SG"], ["zh-Hans"], Some("zh-cmn-Hans-SG"));
		case(["zh", "zh-cmn", "zh-cmn-Hans", "zh-cmn-Hans-SG"], ["zh-Hans"], Some("zh-cmn-Hans"));
		case(["zh", "zh-cmn", "zh-cmn-Hans", "zh-cmn-Hans-SG"], ["zh-SG"], Some("zh-cmn-Hans-SG"));

		// Extensions
		case(["zh", "he"], ["he-IL-u-ca-hebrew-tz-jeruslm", "zh"], Some("he"));
		case(["zh", "he-IL-u-ca-hebrew-tz-jeruslm-nu-latn"], ["he", "zh"], Some("he-IL-u-ca-hebrew-tz-jeruslm-nu-latn"));
		case(["ar-u-nu-latn", "ar"], ["ar-u-no-latn", "ar", "en-US", "en"], Some("ar-u-nu-latn"));
		case(["fr-FR-u-em-text", "gsw-u-em-emoji"], ["gsw-u-em-text"], Some("gsw-u-em-emoji"));

		// Malformed
		case(["en-US-SUS-BUS-VUS-GUS"], ["en"], None);
		case(["en-abcdefghijklmnopqrstuvwxyz"], ["en"], None);
		case(["ru-ЖЖЯЯ"], ["ru"], None);
		case(["ru--"], ["ru"], None);
		case([" en"], ["en"], None);
		case(["", "@", "!!!", "721345"], ["en", "", "@", "!!!", "721345"], None);

		// Repeating
		case(["en", "en", "en", "en"], ["ru-RU", "ru", "en-US", "en"], Some("en"));
		case(["en-US", "en-GB", "ru-UA", "fr-FR", "it"], ["kk", "ru", "pt", "ru"], Some("ru-UA"));

		// Littered
		case(["!!!!!!", "qwydgn12i6i", "ЖЖяяЖяЬЬЬ", "en-US", "!*&^^&*", "qweqweqweqwe-qweqwe", "ru-RU", "@@", "@"], ["ru", "en"], Some("ru-RU"));
		case(["", "", "", "zh", "", "", "", "", "", "he", "", ""], ["he-IL-u-ca-hebrew-tz-jeruslm", "", "", "zh"], Some("he"));
		case(["bla-!@#", "12345", "en-US", "en-GB", "ru-UA", "fr-FR", "it"], ["bla-!@#", "12345", "en-US", "en", "ru-RU", "ru"], Some("en-US"));

		// Special characters
		case(["\0", "\x01", "\x02"], ["\0", "\x01", "\x02"], None);
		case(["en\0"], ["en\0", "en-US", "en"], None);
		case(["sq\0", "ru-RU", "sq-AL", "eu-ES"], ["en-US", "en", "sq-XK", "sq"], Some("sq-AL"));
		case(["en-US", "ru-RU\x03"], ["ru", "en"], Some("en-US"));
		case(["\0", "\x01\x02\x03\x04", "sq\0", "ru-RU", "sq-AL", "eu-ES"], ["en-US", "\x06", "en", "sq-XK", "sq", "\0"], Some("sq-AL"));
		case(["en-US", "ru-RU\x03", "\x09\x09\x09\x09\x09", "\x0a\x09\x08\x07\x01\x00"], ["\x01", "\x02", "\x03", "\x04", "ru", "en"], Some("en-US"));

		// Various letter cases
		case(["EN"], ["en"], Some("EN"));
		case(["En"], ["EN"], Some("En"));
		case(["Ru-rU"], ["en", "ru"], Some("Ru-rU"));
		case(["rU-rU"], ["en", "Ru"], Some("rU-rU"));
		case(["zh", "zh-cmn", "zH-cMn-hANS-Sg"], ["zh-Hans"], Some("zH-cMn-hANS-Sg"));
		case(["zh", "zh-cmn", "zH-cMn-hANS-Sg"], ["ZH-HANS"], Some("zH-cMn-hANS-Sg"));
		case(["zh", "he-IL-u-ca-HEBREW-tz-Jeruslm-nu-LaTn"], ["he", "zh"], Some("he-IL-u-ca-HEBREW-tz-Jeruslm-nu-LaTn"));
		case(["zh", "HE-il-u-cA-HeBrEw-tz-Jeruslm-nu-LaTN"], ["he", "zh"], Some("HE-il-u-cA-HeBrEw-tz-Jeruslm-nu-LaTN"));

		// Various template parameter types
		// &str and &&str
		case(["en-US", "ru-RU"], ["ru", "en"], Some("ru-RU"));
		case(&["en-US", "ru-RU"], ["ru", "en"], Some(&"ru-RU"));
		case(["en-US", "ru-RU"], &["ru", "en"], Some("ru-RU"));
		case(&["en-US", "ru-RU"], &["ru", "en"], Some(&"ru-RU"));
		case([&"en-US", &"ru-RU"], ["ru", "en"], Some(&"ru-RU"));
		// String and &String
		case(["en-US".to_string(), "ru-RU".to_string()], ["ru", "en"], Some("ru-RU".to_string()));
		case(&["en-US".to_string(), "ru-RU".to_string()], ["ru", "en"], Some(&"ru-RU".to_string()));
		// Cow
		use std::borrow::Cow;
		case([Cow::Owned("en-US".to_string()), Cow::Borrowed("ru-RU")], ["ru", "en"], Some(Cow::Borrowed("ru-RU")));
		case([Cow::Borrowed("en-US"), Cow::Owned("ru-RU".to_string())], ["ru", "en"], Some(Cow::Owned("ru-RU".to_string())));
		// Rc and Arc
		use std::rc::Rc;
		use std::sync::Arc;
		case([Rc::from("en-US"), Rc::from("ru-RU")], ["ru", "en"], Some(Rc::from("ru-RU")));
		case([Arc::from("en-US"), Arc::from("ru-RU")], ["ru", "en"], Some(Arc::from("ru-RU")));
		// Box
		case([Box::from("en-US"), Box::from("ru-RU")], ["ru", "en"], Some(Box::from("ru-RU")));
	}
}