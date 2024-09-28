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
/// For example, `best_matching_locale(&["EN"], &["en"])` will return `Some("EN")`.
///
/// # Examples
///
/// ```
/// use locale_match::posix::best_matching_locale;
///
///
/// let available_locales = ["en_US", "en_GB", "ru_UA", "fr_FR", "it"];
/// let user_locales = ["ru_RU", "ru", "en_US", "en"];
///
/// let best_match = best_matching_locale(available_locales, user_locales);
///
/// // "ru_UA" is the best match for the highest-priority user locale "ru_RU"
/// assert_eq!(best_match, Some("ru_UA"));
///
///
/// let available_locales = ["en", "pt_BR", "pt_PT", "es"];
/// let user_locales = ["pt", "en"];
///
/// let best_match = best_matching_locale(available_locales, user_locales);
///
/// // "pt_BR" is the first best match for the highest-priority user locale "pt"
/// assert_eq!(best_match, Some("pt_BR"));
///
///
/// let available_locales = ["fr", "fr_FR", "fr_CA.UTF-8"];
/// let user_locales = ["fr.UTF-8"];
///
/// let best_match = best_matching_locale(available_locales, user_locales);
///
/// // Empty territory in "fr.UTF-8" matches any territory, e.g. "CA"
/// assert_eq!(best_match, Some("fr_CA.UTF-8"));
/// ```
pub fn best_matching_locale<T1, T2>(available_locales: impl IntoIterator<Item = T1>, user_locales: impl IntoIterator<Item = T2>) -> Option<T1>
where
	T1: AsRef<str>,
	T2: AsRef<str>
{
	let available_parsed_locales = available_locales.into_iter()
		.map(|l| PosixLocale::parse(l))
		.collect::<Vec<PosixLocale<T1>>>();

	user_locales.into_iter()
		.map(|locale| PosixLocale::parse(locale))
		.find_map(|user_locale|
			available_parsed_locales.iter()
				.enumerate()
				.rev() // For max_by_key to return the first locale with max score
				.filter(|(_, aval_locale)| aval_locale.language().eq_ignore_ascii_case(user_locale.language()))
				.max_by_key(|(_, aval_locale)| {
					let mut score = 0;
					for (aval, user, weight) in [
						(aval_locale.territory(), user_locale.territory(), 4),
						(aval_locale.codeset(),   user_locale.codeset(),   2),
						(aval_locale.modifier(),  user_locale.modifier(),  1),
					] {
						match (aval, user) {
							(Some(a), Some(u)) if a.eq_ignore_ascii_case(u) => score += weight,
							_ => {} // Ignore if both are None
						}
					}
					score
				})
				.map(|(i, _)| i)
		)
		.map(|i| available_parsed_locales.into_iter().nth(i).unwrap().into_inner())
}

/// A POSIX locale as described in [The Open Group Base Specifications Issue 8 - 8. Environment Variables](https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap08.html).
struct PosixLocale<T: AsRef<str>> {
	locale: T,
	language_end: usize,
	territory_end: usize,
	codeset_end: usize,
}

impl<T: AsRef<str>> PosixLocale<T> {
	const TERRITORY_DELIMITER: char = '_';
	const CODESET_DELIMITER: char = '.';
	const MODIFIER_DELIMITER: char = '@';

	/// Parse a POSIX locale string into a `PosixLocale`.
	///
	/// The `locale` string should be in the form `language[_territory][.codeset][@modifier]`.
	///
	/// The function does not perform any validation on the input string.
	fn parse(locale: T) -> Self {
		let locale_ref = locale.as_ref();
		let codeset_end = locale_ref.find(Self::MODIFIER_DELIMITER).unwrap_or(locale_ref.len());
		let territory_end = locale_ref.find(Self::CODESET_DELIMITER).unwrap_or(codeset_end);
		let language_end = locale_ref.find(Self::TERRITORY_DELIMITER).unwrap_or(territory_end);
		Self { locale, language_end, territory_end, codeset_end }
	}

	fn language(&self) -> &str {
		&self.locale.as_ref()[0..self.language_end]
	}

	fn territory(&self) -> Option<&str> {
		self.locale.as_ref().get(self.language_end + 1..self.territory_end)
	}

	fn codeset(&self) -> Option<&str> {
		self.locale.as_ref().get(self.territory_end + 1..self.codeset_end)
	}

	fn modifier(&self) -> Option<&str> {
		self.locale.as_ref().get(self.codeset_end + 1..)
	}

	fn into_inner(self) -> T {
		self.locale
	}
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
		case(["en_US", "ru_RU"], ["ru", "en"], Some("ru_RU"));
		case(["en_US", "ru_RU"], ["en", "ru"], Some("en_US"));
		case(["en_US", "en_GB", "ru_UA", "fr_FR", "it"], ["ru_RU", "ru", "en_US", "en"], Some("ru_UA"));
		case(["ru_RU", "sq_AL", "eu_ES"], ["en_US", "en", "sq_XK", "sq"], Some("sq_AL"));
		case(["lv_LV", "ru_RU", "lt_LT", "mn_MN", "ku_TR"], ["fr", "fr_FR", "ml", "si", "id", "ku_IQ"], Some("ku_TR"));
		case(["st_LS", "sn_ZW", "en_US"], ["zu_ZA", "st_ZA", "en"], Some("st_LS"));

		// Multiple best matches
		case(["en_US", "en_GB", "ru_UA", "fr_FR", "it"], ["en_US", "en", "ru_RU", "ru"], Some("en_US"));
		case(["en", "pt_BR", "pt_PT", "es"], ["pt", "en"], Some("pt_BR"));
		case(["ku_TR", "ku_IQ", "ku_IR"], ["ku", "en"], Some("ku_TR"));
		case(["en_US", "ru_RU", "mn_CN", "sn_ZW", "en", "ru", "mn_MN", "sn"], ["mn", "ru", "en", "sn"], Some("mn_CN"));

		// Identical
		case(["en"], ["en"], Some("en"));
		case(["en_US"], ["en_US"], Some("en_US"));
		case(["en_US", "ru_RU"], ["en_US", "ru_RU"], Some("en_US"));
		case(["st_LS", "sn_ZW", "en_US"], ["st_LS", "sn_ZW", "en_US"], Some("st_LS"));
		case(["ku_TR", "ku_IQ", "ku_IR"], ["ku_TR", "ku_IQ", "ku_IR"], Some("ku_TR"));
		case(["lv_LV", "ru_RU", "lt_LT", "mn_MN", "ku_TR"], ["lv_LV", "ru_RU", "lt_LT", "mn_MN", "ku_TR"], Some("lv_LV"));

		// More complicated cases
		case(["en_US", "ru_RU.UTF-8"], ["ru", "en"], Some("ru_RU.UTF-8"));
		case(["en_US", "ru.UTF-8", "ru_RU.UTF-8"], ["ru.UTF-8", "en"], Some("ru.UTF-8"));
		case(["en_US", "ru_RU.UTF-8", "ru.UTF-8"], ["ru.UTF-8", "en"], Some("ru_RU.UTF-8"));
		case(["en_US", "ru.UTF-8@dict", "ru_UA"], ["ru_UA.UTF-8@dict", "en"], Some("ru_UA"));
		case(["en_US@dict", "ru_RU"], ["en", "ru"], Some("en_US@dict"));
		case(["en_US.CP1252", "en_GB.UTF-8", "ru_UA@icase", "fr_FR@euro", "it.UTF-8"], ["ru_RU.KOI8-R", "ru@icase", "en_US.UTF-8", "en.CP1252"], Some("ru_UA@icase"));
		case(["fr", "fr_FR", "fr_CA.UTF-8"], ["fr.UTF-8"], Some("fr_CA.UTF-8"));
		case(["en", "pt_BR@dict", "pt_PT@icase", "es"], ["pt.CP1252@euro", "en.UTF-8@dict"], Some("pt_BR@dict"));
		case(["en_US", "ru_RU", "mn_CN.UTF-8", "sn_ZW", "en", "ru", "mn_MN@dict", "sn"], ["mn.UTF-8@dict", "ru", "en", "sn"], Some("mn_CN.UTF-8"));

		// One available locale
		case(["kk"], ["en", "en_US", "fr_FR", "fr", "it", "pt", "ru_RU", "es_ES", "kk_KZ"], Some("kk"));

		// One user locale
		case(["en", "en_US", "fr_FR", "fr", "it", "pt", "ru_RU", "es_ES", "kk_KZ", "pt"], ["pt_PT"], Some("pt"));

		// Not found
		case(["en", "en_US", "fr_FR", "fr", "it", "pt", "es_ES", "kk_KZ", "pt"], ["ru"], None);
		case(["en", "en_US", "fr_FR", "fr", "pt"], ["id"], None);
		case(["ru", "be", "uk", "kk"], ["en"], None);

		// Empty available locales
		case(&[] as &[&str], ["en", "fr", "it", "pt"], None);

		// Empty user locales
		case(["en", "fr", "it", "pt"], &[] as &[&str], None);

		// Both lists empty
		case(&[] as &[&str], &[] as &[&str], None);

		// Malformed
		case([" en"], ["en"], None);
		case(["en\n"], ["en"], None);
		case(["?ru"], ["ru"], None);
		case(["ru!"], ["ru"], None);
		case(["ruRU"], ["ru"], None);

		// Repeating
		case(["en", "en", "en", "en"], ["ru_RU", "ru", "en_US", "en"], Some("en"));
		case(["en_US", "en_GB", "ru_UA", "fr_FR", "it"], ["kk", "ru", "pt", "ru"], Some("ru_UA"));

		// Littered
		case(["!!!!!!", "qwydgn12i6i", "ЖЖяяЖяЬЬЬ", "en_US", "!*&^^&*", "qweqweqweqwe_qweqwe", "ru_RU", "@@", "@"], ["ru", "en"], Some("ru_RU"));
		case(["", "", "", "zh", "", "", "", "", "", "he", "", ""], ["he", "", "", "zh"], Some("he"));

		// Special characters
		case(["sq\0", "ru_RU", "sq_AL", "eu_ES"], ["en_US", "en", "sq_XK", "sq"], Some("sq_AL"));
		case(["\0", "\x01\x02\x03\x04", "sq\0", "ru_RU", "sq_AL", "eu_ES"], &["en_US", "\x06", "en", "sq_XK", "sq", "\0"], Some("sq_AL"));

		// Various letter cases
		case(["EN"], ["en"], Some("EN"));
		case(["En"], ["EN"], Some("En"));
		case(["Ru_rU"], ["en", "ru"], Some("Ru_rU"));
		case(["rU_rU"], ["en", "Ru"], Some("rU_rU"));
		case(["EN.Utf-8"], ["en.UTF-8"], Some("EN.Utf-8"));
		case(["En@dIcT"], ["EN_us"], Some("En@dIcT"));
		case(["ru_ru.utf-8@icase"], ["en", "RU_RU.UTF-8@ICASE"], Some("ru_ru.utf-8@icase"));
		case(["fr_FR.CP1252@euRO"], ["FR", "en"], Some("fr_FR.CP1252@euRO"));

		// Various template parameter types
		// &str and &&str
		case(["en_US", "ru_RU"], ["ru", "en"], Some("ru_RU"));
		case(&["en_US", "ru_RU"], ["ru", "en"], Some(&"ru_RU"));
		case(["en_US", "ru_RU"], &["ru", "en"], Some("ru_RU"));
		case(&["en_US", "ru_RU"], &["ru", "en"], Some(&"ru_RU"));
		case([&"en_US", &"ru_RU"], ["ru", "en"], Some(&"ru_RU"));
		// String and &String
		case(["en_US".to_string(), "ru_RU".to_string()], ["ru", "en"], Some("ru_RU".to_string()));
		case(&["en_US".to_string(), "ru_RU".to_string()], ["ru", "en"], Some(&"ru_RU".to_string()));
		// Cow
		use std::borrow::Cow;
		case([Cow::Owned("en_US".to_string()), Cow::Borrowed("ru_RU")], ["ru", "en"], Some(Cow::Borrowed("ru_RU")));
		case([Cow::Borrowed("en_US"), Cow::Owned("ru_RU".to_string())], ["ru", "en"], Some(Cow::Owned("ru_RU".to_string())));
		// Rc and Arc
		use std::rc::Rc;
		use std::sync::Arc;
		case([Rc::from("en_US"), Rc::from("ru_RU")], ["ru", "en"], Some(Rc::from("ru_RU")));
		case([Arc::from("en_US"), Arc::from("ru_RU")], ["ru", "en"], Some(Arc::from("ru_RU")));
		// Box
		case([Box::from("en_US"), Box::from("ru_RU")], ["ru", "en"], Some(Box::from("ru_RU")));
	}

	#[test]
	#[allow(non_snake_case)]
	fn test_PosixLocale() {

		fn case(locale: &str, parts: (&str, Option<&str>, Option<&str>, Option<&str>)) {
			let posix_locale = PosixLocale::parse(locale);
			assert_eq!(posix_locale.locale, locale);
			assert_eq!(posix_locale.language(), parts.0);
			assert_eq!(posix_locale.territory(), parts.1);
			assert_eq!(posix_locale.codeset(), parts.2);
			assert_eq!(posix_locale.modifier(), parts.3);
		}

		// Language only
		case("en", ("en", None, None, None));
		case("ru", ("ru", None, None, None));
		case("fr", ("fr", None, None, None));

		// Language and territory
		case("en_US", ("en", Some("US"), None, None));
		case("ru_RU", ("ru", Some("RU"), None, None));
		case("fr_FR", ("fr", Some("FR"), None, None));

		// Language and codeset
		case("en.UTF-8", ("en", None, Some("UTF-8"), None));
		case("ru.KOI8-R", ("ru", None, Some("KOI8-R"), None));
		case("fr.CP1252", ("fr", None, Some("CP1252"), None));

		// Language and modifier
		case("en@dict", ("en", None, None, Some("dict")));
		case("ru@icase", ("ru", None, None, Some("icase")));
		case("fr@euro", ("fr", None, None, Some("euro")));

		// Language, territory and codeset
		case("en_US.UTF-8", ("en", Some("US"), Some("UTF-8"), None));
		case("ru_RU.KOI8-R", ("ru", Some("RU"), Some("KOI8-R"), None));
		case("fr_FR.CP1252", ("fr", Some("FR"), Some("CP1252"), None));

		// Language, territory and modifier
		case("en_US@dict", ("en", Some("US"), None, Some("dict")));
		case("ru_RU@icase", ("ru", Some("RU"), None, Some("icase")));
		case("fr_FR@euro", ("fr", Some("FR"), None, Some("euro")));

		// Language, codeset and modifier
		case("en.UTF-8@dict", ("en", None, Some("UTF-8"), Some("dict")));
		case("ru.KOI8-R@icase", ("ru", None, Some("KOI8-R"), Some("icase")));
		case("fr.CP1252@euro", ("fr", None, Some("CP1252"), Some("euro")));

		// Language, territory, codeset and modifier
		case("en_US.UTF-8@dict", ("en", Some("US"), Some("UTF-8"), Some("dict")));
		case("ru_RU.KOI8-R@icase", ("ru", Some("RU"), Some("KOI8-R"), Some("icase")));
		case("fr_FR.CP1252@euro", ("fr", Some("FR"), Some("CP1252"), Some("euro")));

		// Various letter cases
		case("EN", ("EN", None, None, None));
		case("Ru", ("Ru", None, None, None));
		case("fR", ("fR", None, None, None));
		case("eN_us.Utf-8", ("eN", Some("us"), Some("Utf-8"), None));
		case("RU_ru.koi8-R", ("RU", Some("ru"), Some("koi8-R"), None));
		case("Fr_Fr.Cp1252", ("Fr", Some("Fr"), Some("Cp1252"), None));
		case("en_us.utf-8@DICT", ("en", Some("us"), Some("utf-8"), Some("DICT")));
		case("RU_RU.KOI8-R@Icase", ("RU", Some("RU"), Some("KOI8-R"), Some("Icase")));
		case("fR_fR.cP1252@eUrO", ("fR", Some("fR"), Some("cP1252"), Some("eUrO")));

		// Empty
		case("", ("", None, None, None));

		// Whitespace
		case(" ", (" ", None, None, None));
		case("  ", ("  ", None, None, None));
		case("\t", ("\t", None, None, None));
		case("\n", ("\n", None, None, None));
		case("\n  \t\t\n \n\t  \t\t\n\n\t", ("\n  \t\t\n \n\t  \t\t\n\n\t", None, None, None));

		// Litter
		case("!!!", ("!!!", None, None, None));
		case("12345", ("12345", None, None, None));
		case("+-+-", ("+-+-", None, None, None));

		// Malformed
		case("!!!_9999.UUU@()()", ("!!!", Some("9999"), Some("UUU"), Some("()()")));
		case("12_123.1234@12345", ("12", Some("123"), Some("1234"), Some("12345")));
		case("+-+-@+-+-", ("+-+-", None, None, Some("+-+-")));

		// Wrong order EXPECTED TO BE BROKEN
		case("lang.codeset_region@modifier", ("lang.codeset", None, Some("codeset_region"), Some("modifier")));
		case("lang@modifier.codeset_region", ("lang@modifier.codeset", None, None, Some("modifier.codeset_region")));
		case("lang_region@modifier.codeset", ("lang", Some("region@modifier"), None, Some("modifier.codeset")));
		case("lang.codeset@modifier_region", ("lang.codeset@modifier", None, Some("codeset"), Some("modifier_region")));
		case("lang@modifier_region.codeset", ("lang@modifier", Some("region"), None, Some("modifier_region.codeset")));

		// Parts missing
		case("_.@", ("", Some(""), Some(""), Some("")));
		case("_US.UTF-8@dict", ("", Some("US"), Some("UTF-8"), Some("dict")));
		case("ru_.KOI8-R@icase", ("ru", Some(""), Some("KOI8-R"), Some("icase")));
		case("fr_FR.@euro", ("fr", Some("FR"), Some(""), Some("euro")));
		case("de_DE.ISO-8859-1@", ("de", Some("DE"), Some("ISO-8859-1"), Some("")));

		// Special characters
		case("\0", ("\0", None, None, None));
		case("\0_\0.\0@\0", ("\0", Some("\0"), Some("\0"), Some("\0")));
		case("\0\x01\x02\x03", ("\0\x01\x02\x03", None, None, None));
		case("\x03\x02\x01", ("\x03\x02\x01", None, None, None));
	}
}