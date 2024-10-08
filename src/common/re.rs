use crate::truncated;
use anyhow::Result;
use regex::{Regex, RegexBuilder};

const A: &str = "[аa🅰🅰️🇦🇦]";
const I: &str = "[іiI1lℹ️🇮]";
const R: &str = "[рr🇷]";
const D: &str = "[dԁ🇩]";
const O: &str = "[оo0🅾️🇴]";
const P: &str = "[рpρϱ🅿️🇵]";

#[derive(Clone)]
pub struct RegularExpression {
    patterns: [Regex; 2],
}

impl RegularExpression {
    pub async fn new() -> Result<Self> {
        let airdrop = RegexBuilder::new(
            [A, I, R, D, R, O, P]
                .map(|s| s.to_string())
                .join(r"\s?")
                .as_str(),
        )
        .case_insensitive(true)
        .build()?;
        let token_and_wallet = RegexBuilder::new(r"(t|Т)oken.*walle(t|Т)|walle(t|Т).*(t|Т)oken")
            .case_insensitive(true)
            .build()?;
        let patterns = [airdrop, token_and_wallet];

        Ok(Self { patterns })
    }

    pub async fn is_spam(&self, txt: &str) -> Result<bool> {
        let cleaned = Regex::new(r"\s")?.replace_all(txt, " ").to_string();
        let mut result = false;
        for pattern in &self.patterns {
            if pattern.is_match(cleaned.as_str()) {
                result = true;
                break;
            }
        }
        if result {
            log::info!("Message detected as spam by RegularExpression");
            log::debug!("{}", truncated(txt));
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_is_spam() {
        let test_cases = vec![
            ("airdrop", true),
            ("аirdrop", true), // Cyrillic а
            ("аirdrор", true), // Cyrillic а and Cyrillic о
            ("аirdrоp", true), // Cyrillic а and Cyrillic о
            ("аirdrор", true), // Cyrillic а, Cyrillic о, and Cyrillic р
            ("airdrор", true), // Cyrillic о
            ("airdrоp", true), // Cyrillic о
            ("airdrор", true), // Cyrillic о and Cyrillic р
            ("аirdrоp", true), // Cyrillic а, and Cyrillic о
            ("аirdrор", true), // Cyrillic а, Cyrillic о, and Cyrillic р
            ("аirdrop", true), // Cyrillic а
            ("аirdrор", true), // Cyrillic а and Cyrillic р
            ("аirdrop", true), // Cyrillic а
            ("airԁrop", true), // Cyrillic ԁ
            ("aіrԁrop", true), // Cyrillic і and Cyrillic ԁ
            ("airԁroр", true), // Cyrillic р
            ("аirdrор", true), // Cyrillic а and Cyrillic р
            ("aіrdrop", true), // Cyrillic і
            ("аіrdrop", true), // Cyrillic а and Cyrillic і
            ("аіrdrop", true), // Cyrillic а and Cyrillic і
            ("аіrdrop", true), // Cyrillic а and Cyrillic і
            ("aіrdroр", true), // Cyrillic і and Cyrillic р
            ("аіrdroр", true), // Cyrillic а, Cyrillic і, and Cyrillic р
            ("аirdrор", true), // Cyrillic а and Cyrillic р
            ("aіrԁrор", true), // Cyrillic і, Cyrillic ԁ, and Cyrillic р
            ("aіrԁrop", true), // Cyrillic і and Cyrillic ԁ
            ("airdroр", true), // Cyrillic р
            ("airԁrop", true), // Greek delta, Δ
            ("аirdrор", true), // Greek o, ο
            ("аіrԁrop", true), // Greek iota, ι
            ("airԁroр", true), // Greek rho, ρ
            ("аirdrор", true), // Greek omicron, ο
            ("aіrdrop", true), // Greek iota, ι
            ("аіrdrop", true), // Greek alpha, α
            ("аіrdrop", true), // Greek iota, ι
            ("aіrdroр", true), // Greek iota, ι, and rho, ρ
            ("аirdrор", true), // Greek omicron, ο, and rho, ρ
            ("aіrԁrор", true), // Greek iota, ι, delta, Δ, and rho, ρ
            ("aіrԁrop", true), // Greek iota, ι, and delta, Δ
            ("airdroр", true), // Greek rho, ρ
            ("Сlаim  Q СOMMUNITY АIRDROP\n Join the Q movement.", true), // snippet from a real one
            ("🅰irdrop", true), // with emoji
            ("🅰️ℹ️irdr🅾️🇵", true), // with emojis
            ("air drop", true), // with space
            ("a i r d r o p", true), // with single spaces
            ("a i r d r o p", true), // with different kids of spaces
            ("🇦 🇮 🇷 🇩 🇷 🇴 🇵", true), // with special characters and spaces
            ("42", false),
            ("", false),
            ("token", false),
            ("wallet", false),
            ("get your wallet and fill it with free tokens", true),
            ("win many tokens for your new wallet", true),
            ("ask me how to get free tokens\n\nfor your wallet", true),
            ("My walleТs have tokens", true),
        ];
        for (word, expected) in test_cases {
            for w in [word, word.to_uppercase().as_str()] {
                let model = RegularExpression::new().await.unwrap();
                let got = model.is_spam(w).await.unwrap();
                assert_eq!(
                    got, expected,
                    "expected: {:?} for {:?}, got: {:?}",
                    expected, w, got
                );
            }
        }
    }
}
