import { computed, ref } from 'vue'
import { auth_service } from '@/services/auth_service'
import { get_stored_token, remove_stored_token, set_stored_token } from '@/services/api'
import type { LoginRequest, RegisterRequest } from '@/types/auth'

// --- Глобальное состояние (singleton за пределами функции) ---

const token = ref<string | null>(get_stored_token())
const guest_token = ref<string | null>(sessionStorage.getItem('gw_guest_token'))

const solving = ref(false)
const pow_iterations = ref(0)
const error = ref<string | null>(null)

// --- Composable ---

export default function useAuth() {
    const is_authenticated = computed(() => !!token.value)
    const has_guest_access = computed(() => !!guest_token.value)

    const login = async (request: LoginRequest): Promise<void> => {
        error.value = null
        const result = await auth_service.login(request)
        token.value = result.token
        set_stored_token(result.token)
    }

    const register = async (request: RegisterRequest): Promise<void> => {
        error.value = null
        await auth_service.register(request)
    }

    const logout = (): void => {
        token.value = null
        guest_token.value = null
        remove_stored_token()
        sessionStorage.removeItem('gw_guest_token')
    }

    /// Запускает PoW в Web Worker и по завершении получает гостевой токен.
    const get_guest_access = async (): Promise<void> => {
        error.value = null
        solving.value = true
        pow_iterations.value = 0

        try {
            const challenge = await auth_service.get_challenge()
            const nonce = await solve_pow(
                challenge.challenge,
                challenge.difficulty,
                (n) => { pow_iterations.value = n },
            )
            const result = await auth_service.get_guest_token({ challenge: challenge.challenge, nonce })
            guest_token.value = result.token
            sessionStorage.setItem('gw_guest_token', result.token)
        } finally {
            solving.value = false
        }
    }

    return {
        token,
        guest_token,
        is_authenticated,
        has_guest_access,
        solving,
        pow_iterations,
        error,
        login,
        register,
        logout,
        get_guest_access,
    }
}

// --- Вспомогательная функция: запускает Worker и возвращает nonce ---

function solve_pow(
    challenge: string,
    difficulty: number,
    on_progress: (iterations: number) => void,
): Promise<number> {
    return new Promise((resolve, reject) => {
        const worker = new Worker(
            new URL('../workers/pow_worker.ts', import.meta.url),
            { type: 'module' },
        )

        worker.onmessage = (e: MessageEvent<{ nonce?: number; progress?: number }>) => {
            if (e.data.nonce !== undefined) {
                worker.terminate()
                resolve(e.data.nonce)
            } else if (e.data.progress !== undefined) {
                on_progress(e.data.progress)
            }
        }

        worker.onerror = (err) => {
            worker.terminate()
            reject(new Error(err.message))
        }

        worker.postMessage({ challenge, difficulty })
    })
}
