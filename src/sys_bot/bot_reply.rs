// ----- Bot Reply Structure ----- //

pub struct BotReply {
    pub reply_string: String,
    pub is_endcall: bool,
}

impl BotReply {
    pub fn parse_reply(reply: &str) -> Self {
        // Check for command indicators like [endcall]
        let is_endcall = reply.to_lowercase().contains("[endcall]");

        // Clean the reply by removing command indicators
        let mut cleaned_reply = reply
            .replace("[ENDCALL]", "")
            .replace("[endcall]", "")
            .replace("[EndCall]", "")
            .trim()
            .to_string();

        // Strip out the Bot: prefix if it exists
        cleaned_reply = cleaned_reply
            .strip_prefix("BOT:")
            .or_else(|| cleaned_reply.strip_prefix("Bot:"))
            .unwrap_or(&cleaned_reply)
            .trim()
            .to_string();
        
        
        // Return the structured reply
        BotReply {
            reply_string: cleaned_reply,
            is_endcall,
        }
    }
}