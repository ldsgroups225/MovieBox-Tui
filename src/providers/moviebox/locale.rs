use crate::providers::models::MediaDetails;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovieBoxLocale {
    pub language: &'static str,
    pub region: &'static str,
    pub mobile_country_code: &'static str,
    pub timezone: &'static str,
    pub user_agent_locale: &'static str,
}

impl MovieBoxLocale {
    const FRENCH: Self = Self {
        language: "fr",
        region: "FR",
        mobile_country_code: "208",
        timezone: "Europe/Paris",
        user_agent_locale: "fr_FR",
    };

    const ENGLISH: Self = Self {
        language: "en",
        region: "US",
        mobile_country_code: "310",
        timezone: "America/New_York",
        user_agent_locale: "en_US",
    };

    pub fn current() -> Self {
        let configured = std::env::var("MOVIEBOX_LOCALE")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .replace('_', "-");

        match configured.as_str() {
            "en" | "en-us" => Self::ENGLISH,
            _ => Self::FRENCH,
        }
    }

    pub fn matches_language_label(self, label: &str) -> bool {
        let normalized = label.trim().to_lowercase().replace('_', "-");
        match self.language {
            "en" => {
                matches!(normalized.as_str(), "en" | "eng")
                    || normalized.starts_with("en-")
                    || normalized.contains("english")
            }
            _ => {
                matches!(normalized.as_str(), "fr" | "fra" | "fre")
                    || normalized.starts_with("fr-")
                    || normalized.contains("french")
                    || normalized.contains("français")
                    || normalized.contains("francais")
            }
        }
    }

    pub fn caption_sibling_ids(self, details: &MediaDetails) -> Vec<String> {
        let mut ids = Vec::with_capacity(details.dubs.len() + 1);
        let mut seen = std::collections::HashSet::with_capacity(details.dubs.len() + 1);
        if !details.id.value.is_empty() && seen.insert(details.id.value.clone()) {
            ids.push(details.id.value.clone());
        }

        let mut dubs = details.dubs.iter().collect::<Vec<_>>();
        dubs.sort_by_key(|dub| {
            !self.matches_language_label(&format!("{} {}", dub.language, dub.label))
        });
        for dub in dubs {
            if !dub.subject_id.is_empty() && seen.insert(dub.subject_id.clone()) {
                ids.push(dub.subject_id.clone());
            }
        }
        ids
    }
}
