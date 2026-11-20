/// A plate split into visible text and its `|country` suffix.
pub struct LicensePlate<'a> {
    /// Characters shown on the plate, markup tags removed.
    pub text: String,
    /// Country name after `|`, when the plate names one.
    pub country: Option<&'a str>,
}

impl<'a> LicensePlate<'a> {
    /// Parses raw `license_plate` text, markup and country suffix included.
    pub fn parse(raw: &'a str) -> Self {
        let (body, country) = match raw.rsplit_once('|') {
            Some((body, country)) => (body, Some(country.trim())),
            None => (raw, None),
        };
        let mut text = String::with_capacity(body.len());
        let mut in_tag = false;
        for character in body.chars() {
            match character {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => text.push(character),
                _ => {}
            }
        }
        Self {
            text: text.trim().to_owned(),
            country,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_marked_up_plate() {
        let plate = LicensePlate::parse(
            "5L3<offset vshift=-8><img src=/material/ui/lp/czech/$SIDE$b.mat>8438|czech",
        );
        assert_eq!(plate.text, "5L38438");
        assert_eq!(plate.country, Some("czech"));
    }

    #[test]
    fn reads_a_plain_plate() {
        let plate = LicensePlate::parse("37849RP|washington");
        assert_eq!(plate.text, "37849RP");
        assert_eq!(plate.country, Some("washington"));
    }

    #[test]
    fn reads_a_plate_without_a_country() {
        let plate = LicensePlate::parse("TEST");
        assert_eq!(plate.text, "TEST");
        assert_eq!(plate.country, None);
    }
}
