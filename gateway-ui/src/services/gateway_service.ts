import { api_delete, api_get, api_post, api_put } from './api'
import type { GuestServiceInfo, ServiceRequest, ServiceResponse } from '@/types/service'

export const gateway_service = {
    list(): Promise<ServiceResponse[]> {
        return api_get<ServiceResponse[]>('/gateway-admin/services', true)
    },

    list_guest_services(): Promise<GuestServiceInfo[]> {
        return api_get<GuestServiceInfo[]>('/auth/guest-services', false)
    },

    create(request: ServiceRequest): Promise<ServiceResponse> {
        return api_post<ServiceResponse>('/gateway-admin/services', request, true)
    },

    update(id: number, request: ServiceRequest): Promise<ServiceResponse> {
        return api_put<ServiceResponse>(`/gateway-admin/services/${id}`, request, true)
    },

    delete(id: number): Promise<void> {
        return api_delete(`/gateway-admin/services/${id}`, true)
    },
}
