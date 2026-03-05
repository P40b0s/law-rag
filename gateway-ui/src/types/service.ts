export interface GuestServiceInfo {
    id: number
    prefix: string
    service_name: string
}

export interface ServiceResponse {
    id: number
    prefix: string
    service_name: string
    targets: string[]
    permissions: string[]
    allow_guests: boolean
}

export interface ServiceRequest {
    prefix: string
    service_name: string
    targets: string[]
    permissions: string[]
    allow_guests: boolean
}
