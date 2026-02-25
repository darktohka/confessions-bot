// Custom IDs for the buttons
pub const CONFESS_BUTTON_ID: &str = "confess_button";
pub const REPLY_BUTTON_ID: &str = "reply_button";

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

#[derive(Debug, poise::Modal)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct ReplyModal {
    #[name = "Reply Content"]
    #[placeholder = "Submit your anonymous reply here."]
    #[paragraph]
    #[max_length = 2000]
    pub content: String,
}
