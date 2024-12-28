use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use lazy_static::lazy_static; // Allows for defining static variables that are initialized lazily
use ahash::AHashMap; // A fast, non-cryptographic hashmap implementation

type HmacSha256 = Hmac<Sha256>;

lazy_static! {
    static ref SECRET_KEY: Vec<u8> = {
        let bot_token = env::var("BOT_TOKEN")
            .unwrap_or_else(|_| "AAAA:0000".to_string());

        // Create an HMAC instance with the key "WebAppData"
        let mut mac = HmacSha256::new_from_slice(b"WebAppData")
            .expect("Failed to create HMAC instance");

        // Update the HMAC with the bot token
        mac.update(bot_token.as_bytes());
        mac.finalize().into_bytes().to_vec()
    };
}

// `thread_local!` is used to define thread-local buffers to avoid repeated allocations
thread_local! {
    // A buffer for storing key-value pairs during processing
    static PAIRS_BUF: std::cell::RefCell<Vec<(String, String)>> = 
        std::cell::RefCell::new(Vec::with_capacity(10));

    // A buffer for storing the hexadecimal representation of the computed hash
    static HEX_BUF: std::cell::RefCell<[u8; 64]> = 
        std::cell::RefCell::new([0u8; 64]);
}

pub fn validate_init_data(init_data: &str) -> Result<bool, &'static str> {
    if init_data.len() > 1024 {
        return Err("Input data too long");
    }

    // Ensure the input data contains only valid ASCII characters, `&`, or `=`
    if !init_data.chars().all(|c| c.is_ascii() && !c.is_control() || c == '&' || c == '=') {
        return Err("Invalid characters in input");
    }

    // Parse the `init_data` into key-value pairs
    let mut params = AHashMap::with_capacity(10); // Stores the parsed parameters
    let mut received_hash = None; // Stores the "hash" value from the input

    // Split the input string into key-value pairs
    for pair in init_data.split('&') {
        if let Some(sep_idx) = pair.find('=') {
            // Split the pair into key and value
            let (key, value) = pair.split_at(sep_idx);

            if key == "hash" {
                // If the key is "hash", store the value (excluding the '=' character)
                received_hash = Some(&value[1..]);
            } else {
                // Otherwise, insert the key-value pair into the hashmap
                params.insert(key, &value[1..]);
            }
        }
    }

    // Ensure the "hash" parameter was present
    let received_hash = received_hash.ok_or("Missing 'hash' parameter")?;

    // Retrieve and validate the "auth_date" parameter
    let auth_date = params
        .get("auth_date")
        .ok_or("Missing 'auth_date' parameter")? // Ensure "auth_date" exists
        .parse::<u64>() // Parse it as a 64-bit unsigned integer
        .map_err(|_| "Invalid 'auth_date' value")?; // Handle parsing errors

    // Get the current time in seconds since the UNIX epoch
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "System time is before UNIX epoch")? // Handle errors if the system time is invalid
        .as_secs();

    // Check if the "auth_date" is too old (more than 1 hour ago)
    if current_time > auth_date + 3600 {
        return Err("auth_date expired");
    }

    // Use the thread-local buffer for storing sorted key-value pairs
    PAIRS_BUF.with(|buf| {
        let mut pairs = buf.borrow_mut(); // Borrow the buffer
        pairs.clear(); // Clear any existing data in the buffer

        // Add all key-value pairs from the hashmap to the buffer
        for (k, v) in params {
            pairs.push((k.to_string(), v.to_string()));
        }

        // Sort the pairs by key (lexicographically)
        pairs.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        // Construct the data check string by concatenating "key=value" pairs with '\n' as a separator
        let data_check_string = pairs.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        // Create a new HMAC instance using the precomputed secret key
        let mut mac = HmacSha256::new_from_slice(&SECRET_KEY)
            .map_err(|_| "Failed to create HMAC instance")?;
        mac.update(data_check_string.as_bytes()); // Update the HMAC with the data check string
        let hash = mac.finalize().into_bytes(); // Finalize the HMAC and get the resulting hash

        // Use the thread-local buffer for storing the hexadecimal hash
        HEX_BUF.with(|hex_buf| {
            let mut buf = hex_buf.borrow_mut(); // Borrow the buffer

            // Encode the hash as a hexadecimal string and store it in the buffer
            hex::encode_to_slice(&hash, &mut *buf)
                .map_err(|_| "Failed to encode hash")?;

            // Convert the buffer to a UTF-8 string
            let computed_hash = std::str::from_utf8(&*buf)
                .map_err(|_| "Invalid UTF-8 in hash")?;

            // Compare the computed hash with the received hash and return the result
            Ok(computed_hash == received_hash)
        })
    })
}
