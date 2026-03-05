/// Решает Proof-of-Work задачу в фоновом потоке.
///
/// Ищет nonce такой, что hex(SHA-256(challenge + nonce))
/// начинается с `difficulty` нулевых символов.
///
/// Сообщения родителю:
///   { progress: number }   — каждые 2000 итераций
///   { nonce: number }       — найдено решение

const PROGRESS_INTERVAL = 2000

self.onmessage = async (e: MessageEvent<{ challenge: string; difficulty: number }>) => {
    const { challenge, difficulty } = e.data
    const prefix = '0'.repeat(difficulty)
    const encoder = new TextEncoder()

    for (let nonce = 0; ; nonce++) {
        const data = encoder.encode(challenge + nonce)
        const hash_buf = await crypto.subtle.digest('SHA-256', data)
        const hash_hex = Array.from(new Uint8Array(hash_buf))
            .map(b => b.toString(16).padStart(2, '0'))
            .join('')

        if (hash_hex.startsWith(prefix)) {
            self.postMessage({ nonce })
            break
        }

        if (nonce % PROGRESS_INTERVAL === 0 && nonce > 0) {
            self.postMessage({ progress: nonce })
        }
    }
}
