import { api_get, api_post } from './api'
import type { ChallengeResponse, GuestTokenRequest, LoginRequest, RegisterRequest, TokenResponse } from '@/types/auth'

export const auth_service = {
    login(request: LoginRequest): Promise<TokenResponse> {
        return api_post<TokenResponse>('/auth/login', request)
    },

    register(request: RegisterRequest): Promise<void> {
        return api_post<void>('/auth/register', request)
    },

    get_challenge(): Promise<ChallengeResponse> {
        return api_get<ChallengeResponse>('/auth/guest-challenge')
    },

    get_guest_token(request: GuestTokenRequest): Promise<TokenResponse> {
        return api_post<TokenResponse>('/auth/guest-token', request)
    },
}
