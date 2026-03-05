export interface LoginRequest {
    username: string
    password: string
}

export interface RegisterRequest {
    username: string
    password: string
    first_name: string
    second_name: string
    surname: string
}

export interface GuestTokenRequest {
    challenge: string
    nonce: number
}

export interface ChallengeResponse {
    challenge: string
    difficulty: number
}

export interface TokenResponse {
    token: string
}
