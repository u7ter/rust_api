use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier, SaltString},
    Argon2
};

pub fn hash_password(password: &str) -> String {
    // Generate a random salt
    let salt = SaltString::generate(&mut rand::thread_rng());

    // Create an Argon2 instance (using default params)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    argon2.hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string()
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    // Parse the hash
    let parsed_hash = argon2::password_hash::PasswordHash::new(hash)
        .expect("Failed to parse password hash");

    // Verify password against the parsed hash
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
}
