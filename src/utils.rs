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
