use anyhow::Result;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use flate2::read::{GzDecoder, GzEncoder};
use flate2::Compression;
use std::io::Read;

pub struct Decoder;

impl Decoder {
    // 1. URL Encoding / Decoding
    pub fn url_encode(input: &str) -> String {
        urlencoding::encode(input).into_owned()
    }

    pub fn url_decode(input: &str) -> Result<String> {
        Ok(urlencoding::decode(input)?.into_owned())
    }

    // 2. Base64 Standard & URL Safe
    pub fn base64_encode(input: &[u8]) -> String {
        STANDARD.encode(input)
    }

    pub fn base64_decode(input: &str) -> Result<Vec<u8>> {
        let clean = input.trim().replace('\n', "").replace('\r', "");
        Ok(STANDARD.decode(clean)?)
    }

    pub fn base64_url_encode(input: &[u8]) -> String {
        URL_SAFE_NO_PAD.encode(input)
    }

    pub fn base64_url_decode(input: &str) -> Result<Vec<u8>> {
        let clean = input.trim().replace('\n', "").replace('\r', "");
        Ok(URL_SAFE_NO_PAD.decode(clean)?)
    }

    // 3. Hexadecimal
    pub fn hex_encode(input: &[u8]) -> String {
        hex::encode(input)
    }

    pub fn hex_decode(input: &str) -> Result<Vec<u8>> {
        let clean = input.trim().replace(' ', "").replace('\n', "");
        Ok(hex::decode(clean)?)
    }

    // 4. HTML Entities
    pub fn html_escape(input: &str) -> String {
        html_escape::encode_text(input).to_string()
    }

    pub fn html_unescape(input: &str) -> String {
        html_escape::decode_html_entities(input).to_string()
    }

    // 5. Gzip Compression / Decompression
    pub fn gzip_compress(input: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(input, Compression::default());
        let mut buffer = Vec::new();
        encoder.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    pub fn gzip_decompress(input: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = GzDecoder::new(input);
        let mut buffer = Vec::new();
        decoder.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_all_formats() -> Result<()> {
        let text = "Hello, RustProxy! <script>alert('test')</script>";

        // URL
        let url_encoded = Decoder::url_encode(text);
        let url_decoded = Decoder::url_decode(&url_encoded)?;
        assert_eq!(url_decoded, text);

        // Base64
        let b64 = Decoder::base64_encode(text.as_bytes());
        let b64_dec = Decoder::base64_decode(&b64)?;
        assert_eq!(String::from_utf8(b64_dec)?, text);

        // Hex
        let hex_enc = Decoder::hex_encode(text.as_bytes());
        let hex_dec = Decoder::hex_decode(&hex_enc)?;
        assert_eq!(String::from_utf8(hex_dec)?, text);

        // HTML
        let html_esc = Decoder::html_escape(text);
        assert!(!html_esc.contains("<script>"));
        let html_unesc = Decoder::html_unescape(&html_esc);
        assert_eq!(html_unesc, text);

        // Gzip
        let gzipped = Decoder::gzip_compress(text.as_bytes())?;
        let ungzipped = Decoder::gzip_decompress(&gzipped)?;
        assert_eq!(String::from_utf8(ungzipped)?, text);

        Ok(())
    }
}
