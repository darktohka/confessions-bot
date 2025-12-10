// Custom IDs for the button
pub const CONFESS_BUTTON_ID: &str = "confess_button";

#[derive(Debug, poise::Modal)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct ConfessionModal {
    #[name = "Confession Content"]
    #[placeholder = "Remember: All confessions are anonymous."]
    #[paragraph]
    #[max_length = 2000]
    pub content: String,
}

/// Check if a confession contains any blacklisted terms.
/// Returns a list of flagged terms found in the confession (case-insensitive).
pub fn check_blacklist(content: &str, blacklist: &[String]) -> Vec<String> {
    let content_lower = content.to_lowercase();
    let mut flagged_terms = Vec::new();

    for term in blacklist {
        let term_lower = term.to_lowercase();
        if content_lower.contains(&term_lower) {
            flagged_terms.push(term.clone());
        }
    }

    flagged_terms
}
