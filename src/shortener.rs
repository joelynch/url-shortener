use base64ct::{Base64UrlUnpadded, Encoding};
use digest::Digest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShorteningStrategy {
    Sha256 { length: usize },
}

impl ShorteningStrategy {
    pub fn shorten(&self, url: &str) -> String {
        match self {
            ShorteningStrategy::Sha256 { length } => {
                Self::digest_shorten::<sha2::Sha256>(url, *length)
            }
        }
    }

    fn digest_shorten<D: Digest>(url: &str, length: usize) -> String {
        let mut hasher = D::new();
        hasher.update(url.as_bytes());
        let hash = hasher.finalize();
        let mut res = Base64UrlUnpadded::encode_string(&hash);
        res.truncate(length);
        res
    }
}

// We need to be able to generate many shortened urls for the same url.
// This struct just maintains a counter and increments it after each call, and
// appends the counter to the url each time.
// Eg next_shortened() with url == https://www.google.com will return
// digest_shorten(https://www.google.com|0)
// digest_shorten(https://www.google.com|1)
// ...
pub struct Shortener {
    strategy: ShorteningStrategy,
    url: String,
    attempt: u32,
}

impl Shortener {
    pub fn new(strategy: ShorteningStrategy, mut url: String) -> Self {
        url.push('|'); // This character can't be contained in a url
        url.push('0');
        Self {
            strategy,
            url,
            attempt: 0,
        }
    }

    pub fn next_shortened(&mut self) -> String {
        if self.attempt == 0 {
            self.attempt += 1;
            return self.strategy.shorten(&self.url);
        }
        self.url
            .truncate(self.url.len() - (self.attempt - 1).to_string().len());
        self.url.push_str(&self.attempt.to_string());
        self.attempt += 1;
        self.strategy.shorten(&self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_shorten() {
        let strategy = ShorteningStrategy::Sha256 { length: 8 };
        let url = "https://www.google.com|0";
        let shortened = strategy.shorten(url);
        assert_eq!(shortened, "NTQmN-z5");
    }

    #[test]
    fn test_shortener() {
        let strategy = ShorteningStrategy::Sha256 { length: 8 };
        let url = "https://www.google.com";
        let mut shortener = Shortener::new(strategy, url.to_string());
        assert_eq!(shortener.next_shortened(), "NTQmN-z5");
        assert_eq!(shortener.next_shortened(), "YQLGtT3-");
        assert_eq!(shortener.next_shortened(), "wJZTvWsB");
    }
}
