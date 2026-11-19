//! Display forms of save values; commands carry the raw number.

use siir_save::Game;
use siir_save::catalog::Currency;

/// The currency the save stores, for when the catalog has no currency list.
pub fn native_currency(game: Game) -> Currency {
    let (code, sign) = match game {
        Game::Ets2 => ("EUR", "€"),
        Game::Ats => ("USD", "$"),
    };
    Currency {
        code: code.to_owned(),
        ratio: 1.0,
        sign1: String::new(),
        sign2: sign.to_owned(),
        sign3: String::new(),
    }
}

/// The game's thousands separator.
pub fn grouping(game: Game) -> char {
    match game {
        Game::Ets2 => ' ',
        Game::Ats => ',',
    }
}

/// A grouped unsigned number.
pub fn number(value: u64, game: Game) -> String {
    let separator = grouping(game);
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (ix, digit) in digits.chars().enumerate() {
        if ix > 0 && (digits.len() - ix).is_multiple_of(3) {
            out.push(separator);
        }
        out.push(digit);
    }
    out
}

/// A grouped signed number.
pub fn amount(value: i64, game: Game) -> String {
    let digits = number(value.unsigned_abs(), game);
    if value < 0 {
        format!("-{digits}")
    } else {
        digits
    }
}

/// An amount the save stores, converted to `currency` and laid out the way
/// the game shows it: `[sign1]-[sign2]1 234[sign3]`.
pub fn money(native: i64, currency: &Currency, game: Game) -> String {
    let shown = currency.from_native(native);
    let minus = if shown < 0 { "-" } else { "" };
    format!(
        "{}{minus}{}{}{}",
        currency.sign1,
        currency.sign2,
        number(shown.unsigned_abs(), game),
        currency.sign3
    )
}

/// A 0 to 1 fraction as a whole percentage.
pub fn percent(fraction: f32) -> String {
    format!("{}%", (fraction.clamp(0.0, 1.0) * 100.0).round())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_numbers_per_game() {
        assert_eq!(number(0, Game::Ets2), "0");
        assert_eq!(number(999, Game::Ats), "999");
        assert_eq!(number(700_047, Game::Ets2), "700 047");
        assert_eq!(number(6_499, Game::Ats), "6,499");
        assert_eq!(number(u64::MAX, Game::Ats), "18,446,744,073,709,551,615");
    }

    #[test]
    fn rounds_fractions_to_percent() {
        assert_eq!(percent(0.0), "0%");
        assert_eq!(percent(0.125), "13%");
        assert_eq!(percent(1.0), "100%");
        assert_eq!(percent(1.5), "100%");
        assert_eq!(percent(-0.2), "0%");
    }

    #[test]
    fn groups_amounts_per_game() {
        assert_eq!(amount(99_738_872, Game::Ets2), "99 738 872");
        assert_eq!(amount(99_738_872, Game::Ats), "99,738,872");
        assert_eq!(amount(2_000, Game::Ats), "2,000");
        assert_eq!(amount(-1_234, Game::Ats), "-1,234");
        assert_eq!(amount(-999, Game::Ets2), "-999");
        assert_eq!(amount(i64::MIN, Game::Ats), "-9,223,372,036,854,775,808");
        assert_eq!(
            money(2_000, &native_currency(Game::Ets2), Game::Ets2),
            "€2 000"
        );
        assert_eq!(
            money(2_000, &native_currency(Game::Ats), Game::Ats),
            "$2,000"
        );
    }

    #[test]
    fn lays_money_out_like_the_game() {
        let koruna = Currency {
            code: "CZK".to_owned(),
            ratio: 24.4,
            sign1: String::new(),
            sign2: String::new(),
            sign3: " Kč".to_owned(),
        };
        assert_eq!(money(1_000, &koruna, Game::Ets2), "24 400 Kč");
        assert_eq!(money(-1_000, &koruna, Game::Ets2), "-24 400 Kč");
        let euro = native_currency(Game::Ets2);
        assert_eq!(money(-5, &euro, Game::Ets2), "-€5");
    }
}
