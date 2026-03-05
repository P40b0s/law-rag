/// Proof-of-Work: SHA-256 hashcash схема.
///
/// Клиент должен найти `nonce` такой, что
/// `hex(SHA-256(challenge + nonce))` начинается с `DIFFICULTY` нулевых hex-символов.
/// При difficulty=4 → в среднем ~65 536 итераций (≈50–200 мс в браузере).

use sha2::{Digest, Sha256};

/// Количество ведущих нулевых hex-символов, которые требуются в хеше.
pub const DIFFICULTY: usize = 4;

/// Максимальное время жизни challenge (секунды).
pub const CHALLENGE_TTL_SECS: u64 = 600; // 10 минут

/// Время жизни гостевого токена (минуты).
pub const GUEST_TOKEN_LIFETIME_MIN: i64 = 5;

/// Генерирует случайный challenge — 32 байта в hex (64 символа).
pub fn generate_challenge() -> String
{
    let bytes: [u8; 32] = rand::random();
    hex::encode(bytes)
}

/// Проверяет, является ли `nonce` решением для данного `challenge`.
///
/// Возвращает `true` если `hex(SHA-256(challenge + nonce))`
/// начинается с `DIFFICULTY` нулевых символов.
pub fn verify(challenge: &str, nonce: u64) -> bool
{
    let input = format!("{}{}", challenge, nonce);
    let hash = Sha256::digest(input.as_bytes());
    let hex_hash = hex::encode(hash);
    hex_hash.starts_with(&"0".repeat(DIFFICULTY))
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn verify_finds_valid_nonce()
    {
        let challenge = generate_challenge();
        // Брут-форсим вручную, чтобы убедиться что verify работает
        let prefix = "0".repeat(DIFFICULTY);
        for nonce in 0u64..
        {
            let input = format!("{}{}", challenge, nonce);
            let hash = hex::encode(Sha256::digest(input.as_bytes()));
            if hash.starts_with(&prefix)
            {
                assert!(verify(&challenge, nonce));
                break;
            }
        }
    }
}
