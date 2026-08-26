/// Public URL of the human-facing inspector for a token.
///
/// The inspector is a single page that reads its token from the URL fragment, so the
/// token has to be handed over as `#<token>` — a path segment renders an empty session.
/// `/view/{token}` next to it is the JSON view of the same session, meant for programs.
pub fn inspector_url(base_url: &str, token: &str) -> String {
    format!("{}/#{}", base_url.trim_end_matches('/'), token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspector_url_hands_the_token_over_as_a_fragment() {
        assert_eq!(
            inspector_url("https://play.hook0.com", "c_abc"),
            "https://play.hook0.com/#c_abc"
        );
    }

    #[test]
    fn inspector_url_does_not_double_the_slash_of_a_base_url_ending_with_one() {
        assert_eq!(
            inspector_url("https://play.hook0.com/", "c_abc"),
            "https://play.hook0.com/#c_abc"
        );
    }
}
