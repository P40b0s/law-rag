const TOKEN_KEY = 'gw_token'

export function get_stored_token(): string | null {
    return localStorage.getItem(TOKEN_KEY)
}

export function set_stored_token(token: string): void {
    localStorage.setItem(TOKEN_KEY, token)
}

export function remove_stored_token(): void {
    localStorage.removeItem(TOKEN_KEY)
}

export class ApiError extends Error {
    constructor(public status: number, message: string) {
        super(message)
        this.name = 'ApiError'
    }
}

async function handle_response<T>(response: Response): Promise<T> {
    if (!response.ok) {
        let message = `HTTP ${response.status}`
        try {
            const body = await response.text()
            // Backend returns {"err": "..."} or plain text
            const parsed = JSON.parse(body)
            message = parsed.err ?? body
        } catch {
            // plain text response, ignore parse error
        }
        throw new ApiError(response.status, message)
    }
    const content_type = response.headers.get('Content-Type') ?? ''
    if (content_type.includes('application/json')) {
        return response.json() as Promise<T>
    }
    return {} as T
}

export async function api_get<T>(path: string, auth = false): Promise<T> {
    const headers: Record<string, string> = {}
    if (auth) {
        const token = get_stored_token()
        if (token) headers['Authorization'] = `Bearer ${token}`
    }
    const response = await fetch(path, { headers })
    return handle_response<T>(response)
}

export async function api_post<T>(path: string, body?: unknown, auth = false): Promise<T> {
    const headers: Record<string, string> = {}
    if (body !== undefined) headers['Content-Type'] = 'application/json'
    if (auth) {
        const token = get_stored_token()
        if (token) headers['Authorization'] = `Bearer ${token}`
    }
    const response = await fetch(path, {
        method: 'POST',
        headers,
        body: body !== undefined ? JSON.stringify(body) : undefined,
    })
    return handle_response<T>(response)
}

export async function api_put<T>(path: string, body: unknown, auth = true): Promise<T> {
    const headers: Record<string, string> = { 'Content-Type': 'application/json' }
    if (auth) {
        const token = get_stored_token()
        if (token) headers['Authorization'] = `Bearer ${token}`
    }
    const response = await fetch(path, {
        method: 'PUT',
        headers,
        body: JSON.stringify(body),
    })
    return handle_response<T>(response)
}

export async function api_delete(path: string, auth = true): Promise<void> {
    const headers: Record<string, string> = {}
    if (auth) {
        const token = get_stored_token()
        if (token) headers['Authorization'] = `Bearer ${token}`
    }
    const response = await fetch(path, { method: 'DELETE', headers })
    if (!response.ok) {
        await handle_response(response)
    }
}
