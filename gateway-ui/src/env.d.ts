/// <reference types="@rsbuild/core/types" />

interface ImportMetaEnv {
    readonly PUBLIC_GATEWAY_PORT?: string
    readonly PUBLIC_GATEWAY_HOST?: string
    /** URL основного приложения — куда перенаправлять после логина */
    readonly PUBLIC_MAIN_APP_URL?: string
}

interface ImportMeta {
    readonly env: ImportMetaEnv
}
