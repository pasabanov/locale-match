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

/// Finds the best matching locale from a list of available locales based on a list of user locales.
/// The function expects locales to be valid POSIX locales, but does not validate them.
/// The function expects locales to be encoded with ASCII.
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
/// For example, `best_matching_locale(&["EN"].iter(), &["en"].iter())` will return `Some("EN")`.
///
/// # Examples
///
/// ```
/// use locale_match::posix::best_matching_locale;
///
///
/// let available_locales = vec!["en_US", "en_GB", "ru_UA", "fr_FR", "it"];
/// let user_locales = vec!["ru_RU", "ru", "en_US", "en"];
///
/// let best_match = best_matching_locale(available_locales.iter(), user_locales.iter());
///
/// // "ru_UA" is the best match for the highest-priority user locale "ru_RU"
/// assert_eq!(best_match, Some("ru_UA"));
///
///
/// let available_locales = vec!["en", "pt_BR", "pt_PT", "es"];
/// let user_locales = vec!["pt", "en"];
///
/// let best_match = best_matching_locale(available_locales.iter(), user_locales.iter());
///
/// // "pt_BR" is the first best match for the highest-priority user locale "pt"
/// assert_eq!(best_match, Some("pt_BR"));
///
///
/// let available_locales = vec!["fr", "fr_FR", "fr_CA.UTF-8"];
/// let user_locales = vec!["fr.UTF-8"];
///
/// let best_match = best_matching_locale(available_locales.iter(), user_locales.iter());
///
/// // Empty territory in "fr.UTF-8" matches any territory, e.g. "CA"
/// assert_eq!(best_match, Some("fr_CA.UTF-8"));
/// ```
pub fn best_matching_locale<'a, 'b, T1, T2>(available_locales: impl Iterator<Item = &'a T1>, user_locales: impl Iterator<Item = &'b T2>) -> Option<&'a str>
where
	T1: AsRef<str> + 'a,
	T2: AsRef<str> + 'b
{
	let available_parsed_locales = available_locales
		.map(|l| PosixLocale::parse(l.as_ref()))
		.collect::<Vec<PosixLocale>>();

	user_locales
		.map(|locale| PosixLocale::parse(locale.as_ref()))
		.find_map(|user_locale|
			available_parsed_locales.iter()
				.rev() // For max_by_key to return the first locale with max score
				.filter(|aval_locale| aval_locale.language.eq_ignore_ascii_case(user_locale.language))
				.max_by_key(|aval_locale| {
					let mut score = 0;
					for (aval, user, weight) in [
						(aval_locale.territory, user_locale.territory, 4),
						(aval_locale.codeset,   user_locale.codeset,   2),
						(aval_locale.modifier,  user_locale.modifier,  1),
					] {
						match (aval, user) {
							(Some(a), Some(u)) if a.eq_ignore_ascii_case(u) => score += weight,
							_ => {} // Ignore if both are None
						}
					}
					score
				})
		)
		.map(|aval_locale| aval_locale.locale)
}

/// A POSIX locale as described in [The Open Group Base Specifications Issue 8 - 8. Environment Variables](https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap08.html).
struct PosixLocale<'a> {
	locale: &'a str,
	language: &'a str,
	territory: Option<&'a str>,
	codeset: Option<&'a str>,
	modifier: Option<&'a str>,
}

impl<'a> PosixLocale<'a> {
	const TERRITORY_DELIMITER: char = '_';
	const CODESET_DELIMITER: char = '.';
	const MODIFIER_DELIMITER: char = '@';

	/// Parse a POSIX locale string into a `PosixLocale`.
	///
	/// The `locale` string should be in the form `language[_territory][.codeset][@modifier]`.
	///
	/// The function does not perform any validation on the input string.
	fn parse(locale: &'a str) -> Self {
		let codeset_end = locale.find(Self::MODIFIER_DELIMITER).unwrap_or(locale.len());
		let territory_end = locale.find(Self::CODESET_DELIMITER).unwrap_or(codeset_end);
		let language_end = locale.find(Self::TERRITORY_DELIMITER).unwrap_or(territory_end);
		Self {
			locale,
			language: &locale[..language_end],
			territory: locale.get(language_end + 1..territory_end),
			codeset: locale.get(territory_end + 1..codeset_end),
			modifier: locale.get(codeset_end + 1..)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_best_matching_locale() {

		fn assert_best_match(available_locales: &[&str], user_locales: &[&str], expected: Option<&str>) {
			assert_eq!(best_matching_locale(available_locales.iter(), user_locales.iter()).as_deref(), expected);
		}

		// One best match
		assert_best_match(&["en_US", "ru_RU"], &["ru", "en"], Some("ru_RU"));
		assert_best_match(&["en_US", "ru_RU"], &["en", "ru"], Some("en_US"));
		assert_best_match(&["en_US", "en_GB", "ru_UA", "fr_FR", "it"], &["ru_RU", "ru", "en_US", "en"], Some("ru_UA"));
		assert_best_match(&["ru_RU", "sq_AL", "eu_ES"], &["en_US", "en", "sq_XK", "sq"], Some("sq_AL"));
		assert_best_match(&["lv_LV", "ru_RU", "lt_LT", "mn_MN", "ku_TR"], &["fr", "fr_FR", "ml", "si", "id", "ku_IQ"], Some("ku_TR"));
		assert_best_match(&["st_LS", "sn_ZW", "en_US"], &["zu_ZA", "st_ZA", "en"], Some("st_LS"));

		// Multiple best matches
		assert_best_match(&["en_US", "en_GB", "ru_UA", "fr_FR", "it"], &["en_US", "en", "ru_RU", "ru"], Some("en_US"));
		assert_best_match(&["en", "pt_BR", "pt_PT", "es"], &["pt", "en"], Some("pt_BR"));
		assert_best_match(&["ku_TR", "ku_IQ", "ku_IR"], &["ku", "en"], Some("ku_TR"));
		assert_best_match(&["en_US", "ru_RU", "mn_CN", "sn_ZW", "en", "ru", "mn_MN", "sn"], &["mn", "ru", "en", "sn"], Some("mn_CN"));

		// Identical
		assert_best_match(&["en"], &["en"], Some("en"));
		assert_best_match(&["en_US"], &["en_US"], Some("en_US"));
		assert_best_match(&["en_US", "ru_RU"], &["en_US", "ru_RU"], Some("en_US"));
		assert_best_match(&["st_LS", "sn_ZW", "en_US"], &["st_LS", "sn_ZW", "en_US"], Some("st_LS"));
		assert_best_match(&["ku_TR", "ku_IQ", "ku_IR"], &["ku_TR", "ku_IQ", "ku_IR"], Some("ku_TR"));
		assert_best_match(&["lv_LV", "ru_RU", "lt_LT", "mn_MN", "ku_TR"], &["lv_LV", "ru_RU", "lt_LT", "mn_MN", "ku_TR"], Some("lv_LV"));

		// More complicated cases
		assert_best_match(&["en_US", "ru_RU.UTF-8"], &["ru", "en"], Some("ru_RU.UTF-8"));
		assert_best_match(&["en_US", "ru.UTF-8", "ru_RU.UTF-8"], &["ru.UTF-8", "en"], Some("ru.UTF-8"));
		assert_best_match(&["en_US", "ru_RU.UTF-8", "ru.UTF-8"], &["ru.UTF-8", "en"], Some("ru_RU.UTF-8"));
		assert_best_match(&["en_US", "ru.UTF-8@dict", "ru_UA"], &["ru_UA.UTF-8@dict", "en"], Some("ru_UA"));
		assert_best_match(&["en_US@dict", "ru_RU"], &["en", "ru"], Some("en_US@dict"));
		assert_best_match(&["en_US.CP1252", "en_GB.UTF-8", "ru_UA@icase", "fr_FR@euro", "it.UTF-8"], &["ru_RU.KOI8-R", "ru@icase", "en_US.UTF-8", "en.CP1252"], Some("ru_UA@icase"));
		assert_best_match(&["fr", "fr_FR", "fr_CA.UTF-8"], &["fr.UTF-8"], Some("fr_CA.UTF-8"));
		assert_best_match(&["en", "pt_BR@dict", "pt_PT@icase", "es"], &["pt.CP1252@euro", "en.UTF-8@dict"], Some("pt_BR@dict"));
		assert_best_match(&["en_US", "ru_RU", "mn_CN.UTF-8", "sn_ZW", "en", "ru", "mn_MN@dict", "sn"], &["mn.UTF-8@dict", "ru", "en", "sn"], Some("mn_CN.UTF-8"));

		// One available locale
		assert_best_match(&["kk"], &["en", "en_US", "fr_FR", "fr", "it", "pt", "ru_RU", "es_ES", "kk_KZ"], Some("kk"));

		// One user locale
		assert_best_match(&["en", "en_US", "fr_FR", "fr", "it", "pt", "ru_RU", "es_ES", "kk_KZ", "pt"], &["pt_PT"], Some("pt"));

		// Not found
		assert_best_match(&["en", "en_US", "fr_FR", "fr", "it", "pt", "es_ES", "kk_KZ", "pt"], &["ru"], None);
		assert_best_match(&["en", "en_US", "fr_FR", "fr", "pt"], &["id"], None);
		assert_best_match(&["ru", "be", "uk", "kk"], &["en"], None);

		// Empty available locales
		assert_best_match(&[], &["en", "fr", "it", "pt"], None);

		// Empty user locales
		assert_best_match(&["en", "fr", "it", "pt"], &[], None);

		// Both lists empty
		assert_best_match(&[], &[], None);

		// Malformed
		assert_best_match(&[" en"], &["en"], None);
		assert_best_match(&["en\n"], &["en"], None);
		assert_best_match(&["?ru"], &["ru"], None);
		assert_best_match(&["ru!"], &["ru"], None);
		assert_best_match(&["ruRU"], &["ru"], None);

		// Repeating
		assert_best_match(&["en", "en", "en", "en"], &["ru_RU", "ru", "en_US", "en"], Some("en"));
		assert_best_match(&["en_US", "en_GB", "ru_UA", "fr_FR", "it"], &["kk", "ru", "pt", "ru"], Some("ru_UA"));

		// Littered
		assert_best_match(&["!!!!!!", "qwydgn12i6i", "ЖЖяяЖяЬЬЬ", "en_US", "!*&^^&*", "qweqweqweqwe_qweqwe", "ru_RU", "@@", "@"], &["ru", "en"], Some("ru_RU"));
		assert_best_match(&["", "", "", "zh", "", "", "", "", "", "he", "", ""], &["he", "", "", "zh"], Some("he"));

		// Special characters
		assert_best_match(&["sq\0", "ru_RU", "sq_AL", "eu_ES"], &["en_US", "en", "sq_XK", "sq"], Some("sq_AL"));
		assert_best_match(&["\0", "\x01\x02\x03\x04", "sq\0", "ru_RU", "sq_AL", "eu_ES"], &["en_US", "\x06", "en", "sq_XK", "sq", "\0"], Some("sq_AL"));

		// Various letter cases
		assert_best_match(&["EN"], &["en"], Some("EN"));
		assert_best_match(&["En"], &["EN"], Some("En"));
		assert_best_match(&["Ru_rU"], &["en", "ru"], Some("Ru_rU"));
		assert_best_match(&["rU_rU"], &["en", "Ru"], Some("rU_rU"));
		assert_best_match(&["EN.Utf-8"], &["en.UTF-8"], Some("EN.Utf-8"));
		assert_best_match(&["En@dIcT"], &["EN_us"], Some("En@dIcT"));
		assert_best_match(&["ru_ru.utf-8@icase"], &["en", "RU_RU.UTF-8@ICASE"], Some("ru_ru.utf-8@icase"));
		assert_best_match(&["fr_FR.CP1252@euRO"], &["FR", "en"], Some("fr_FR.CP1252@euRO"));
	}

	#[test]
	#[allow(non_snake_case)]
	fn test_PosixLocale() {

		fn assert_parts(locale: &str, parts: (&str, Option<&str>, Option<&str>, Option<&str>)) {
			let posix_locale = PosixLocale::parse(locale);
			assert_eq!(posix_locale.locale, locale);
			assert_eq!(posix_locale.language, parts.0);
			assert_eq!(posix_locale.territory, parts.1);
			assert_eq!(posix_locale.codeset, parts.2);
			assert_eq!(posix_locale.modifier, parts.3);
		}

		// Language only
		assert_parts("en", ("en", None, None, None));
		assert_parts("ru", ("ru", None, None, None));
		assert_parts("fr", ("fr", None, None, None));

		// Language and territory
		assert_parts("en_US", ("en", Some("US"), None, None));
		assert_parts("ru_RU", ("ru", Some("RU"), None, None));
		assert_parts("fr_FR", ("fr", Some("FR"), None, None));

		// Language and codeset
		assert_parts("en.UTF-8", ("en", None, Some("UTF-8"), None));
		assert_parts("ru.KOI8-R", ("ru", None, Some("KOI8-R"), None));
		assert_parts("fr.CP1252", ("fr", None, Some("CP1252"), None));

		// Language and modifier
		assert_parts("en@dict", ("en", None, None, Some("dict")));
		assert_parts("ru@icase", ("ru", None, None, Some("icase")));
		assert_parts("fr@euro", ("fr", None, None, Some("euro")));

		// Language, territory and codeset
		assert_parts("en_US.UTF-8", ("en", Some("US"), Some("UTF-8"), None));
		assert_parts("ru_RU.KOI8-R", ("ru", Some("RU"), Some("KOI8-R"), None));
		assert_parts("fr_FR.CP1252", ("fr", Some("FR"), Some("CP1252"), None));

		// Language, territory and modifier
		assert_parts("en_US@dict", ("en", Some("US"), None, Some("dict")));
		assert_parts("ru_RU@icase", ("ru", Some("RU"), None, Some("icase")));
		assert_parts("fr_FR@euro", ("fr", Some("FR"), None, Some("euro")));

		// Language, codeset and modifier
		assert_parts("en.UTF-8@dict", ("en", None, Some("UTF-8"), Some("dict")));
		assert_parts("ru.KOI8-R@icase", ("ru", None, Some("KOI8-R"), Some("icase")));
		assert_parts("fr.CP1252@euro", ("fr", None, Some("CP1252"), Some("euro")));

		// Language, territory, codeset and modifier
		assert_parts("en_US.UTF-8@dict", ("en", Some("US"), Some("UTF-8"), Some("dict")));
		assert_parts("ru_RU.KOI8-R@icase", ("ru", Some("RU"), Some("KOI8-R"), Some("icase")));
		assert_parts("fr_FR.CP1252@euro", ("fr", Some("FR"), Some("CP1252"), Some("euro")));

		// Various letter cases
		assert_parts("EN", ("EN", None, None, None));
		assert_parts("Ru", ("Ru", None, None, None));
		assert_parts("fR", ("fR", None, None, None));
		assert_parts("eN_us.Utf-8", ("eN", Some("us"), Some("Utf-8"), None));
		assert_parts("RU_ru.koi8-R", ("RU", Some("ru"), Some("koi8-R"), None));
		assert_parts("Fr_Fr.Cp1252", ("Fr", Some("Fr"), Some("Cp1252"), None));
		assert_parts("en_us.utf-8@DICT", ("en", Some("us"), Some("utf-8"), Some("DICT")));
		assert_parts("RU_RU.KOI8-R@Icase", ("RU", Some("RU"), Some("KOI8-R"), Some("Icase")));
		assert_parts("fR_fR.cP1252@eUrO", ("fR", Some("fR"), Some("cP1252"), Some("eUrO")));

		// Empty
		assert_parts("", ("", None, None, None));

		// Whitespace
		assert_parts(" ", (" ", None, None, None));
		assert_parts("  ", ("  ", None, None, None));
		assert_parts("\t", ("\t", None, None, None));
		assert_parts("\n", ("\n", None, None, None));
		assert_parts("\n  \t\t\n \n\t  \t\t\n\n\t", ("\n  \t\t\n \n\t  \t\t\n\n\t", None, None, None));

		// Litter
		assert_parts("!!!", ("!!!", None, None, None));
		assert_parts("12345", ("12345", None, None, None));
		assert_parts("+-+-", ("+-+-", None, None, None));

		// Malformed
		assert_parts("!!!_9999.UUU@()()", ("!!!", Some("9999"), Some("UUU"), Some("()()")));
		assert_parts("12_123.1234@12345", ("12", Some("123"), Some("1234"), Some("12345")));
		assert_parts("+-+-@+-+-", ("+-+-", None, None, Some("+-+-")));

		// Wrong order EXPECTED TO BE BROKEN
		assert_parts("lang.codeset_region@modifier", ("lang.codeset", None, Some("codeset_region"), Some("modifier")));
		assert_parts("lang@modifier.codeset_region", ("lang@modifier.codeset", None, None, Some("modifier.codeset_region")));
		assert_parts("lang_region@modifier.codeset", ("lang", Some("region@modifier"), None, Some("modifier.codeset")));
		assert_parts("lang.codeset@modifier_region", ("lang.codeset@modifier", None, Some("codeset"), Some("modifier_region")));
		assert_parts("lang@modifier_region.codeset", ("lang@modifier", Some("region"), None, Some("modifier_region.codeset")));

		// Parts missing
		assert_parts("_.@", ("", Some(""), Some(""), Some("")));
		assert_parts("_US.UTF-8@dict", ("", Some("US"), Some("UTF-8"), Some("dict")));
		assert_parts("ru_.KOI8-R@icase", ("ru", Some(""), Some("KOI8-R"), Some("icase")));
		assert_parts("fr_FR.@euro", ("fr", Some("FR"), Some(""), Some("euro")));
		assert_parts("de_DE.ISO-8859-1@", ("de", Some("DE"), Some("ISO-8859-1"), Some("")));

		// Special characters
		assert_parts("\0", ("\0", None, None, None));
		assert_parts("\0_\0.\0@\0", ("\0", Some("\0"), Some("\0"), Some("\0")));
		assert_parts("\0\x01\x02\x03", ("\0\x01\x02\x03", None, None, None));
		assert_parts("\x03\x02\x01", ("\x03\x02\x01", None, None, None));
	}
}