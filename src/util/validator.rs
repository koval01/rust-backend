use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::env;
use lazy_static::lazy_static;
use ahash::AHashMap;

type HmacSha256 = Hmac<Sha256>;

lazy_static! {
    static ref SECRET_KEY: Vec<u8> = {
        let bot_token = env::var("BOT_TOKEN")
            .expect("BOT_TOKEN must be set in the environment");
        let mut mac = HmacSha256::new_from_slice(b"WebAppData")
            .expect("Failed to create HMAC instance");
        mac.update(bot_token.as_bytes());
        mac.finalize().into_bytes().to_vec()
    };
}

thread_local! {
    static PAIRS_BUF: std::cell::RefCell<Vec<(String, String)>> = 
        std::cell::RefCell::new(Vec::with_capacity(10));
    static HEX_BUF: std::cell::RefCell<[u8; 64]> = 
        std::cell::RefCell::new([0u8; 64]);
}

pub fn validate_init_data(init_data: &str) -> Result<bool, &'static str> {
    if init_data.len() > 768 {
        return Err("Input data too long");
    }

    if !init_data.chars().all(|c| c.is_ascii() && !c.is_control() || c == '&' || c == '=') {
        return Err("Invalid characters in input");
    }
    
    let mut params = AHashMap::with_capacity(10);
    let mut received_hash = None;

    for pair in init_data.split('&') {
        if let Some(sep_idx) = pair.find('=') {
            let (key, value) = pair.split_at(sep_idx);
            if key == "hash" {
                received_hash = Some(&value[1..]);
            } else {
                params.insert(key, &value[1..]);
            }
        }
    }

    let received_hash = received_hash.ok_or("Missing 'hash' parameter")?;

    PAIRS_BUF.with(|buf| {
        let mut pairs = buf.borrow_mut();
        pairs.clear();

        for (k, v) in params {
            pairs.push((k.to_string(), v.to_string()));
        }

        pairs.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let data_check_string = pairs.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        let mut mac = HmacSha256::new_from_slice(&SECRET_KEY)
            .map_err(|_| "Failed to create HMAC instance")?;
        mac.update(data_check_string.as_bytes());
        let hash = mac.finalize().into_bytes();

        HEX_BUF.with(|hex_buf| {
            let mut buf = hex_buf.borrow_mut();
            // Dereference RefMut to get &mut [u8; 64]
            hex::encode_to_slice(&hash, &mut *buf)
                .map_err(|_| "Failed to encode hash")?;

            // Dereference RefMut to get &[u8; 64]
            let computed_hash = std::str::from_utf8(&*buf)
                .map_err(|_| "Invalid UTF-8 in hash")?;

            Ok(computed_hash == received_hash)
        })
    })
}
