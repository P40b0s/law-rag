<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
    NCard, NList, NListItem, NThing, NButton, NSpace,
    NText, NSpin, NAlert, NEmpty,
} from 'naive-ui'
import { gateway_service } from '@/services/gateway_service'
import type { GuestServiceInfo } from '@/types/service'

const loading = ref(true)
const error = ref<string | null>(null)
const services = ref<GuestServiceInfo[]>([])

onMounted(async () => {
    try {
        services.value = await gateway_service.list_guest_services()
    } catch (e: any) {
        error.value = e.message
    } finally {
        loading.value = false
    }
})

const navigate_to = (prefix: string) => {
    window.location.href = `/${prefix}/`
}
</script>

<template lang="pug">
.guest-services-page
    n-card.services-card(title="Доступные сервисы")
        n-text(depth="3" style="display: block; margin-bottom: 16px")
            | Ваш гостевой доступ активен. Выберите сервис для работы.

        n-spin(v-if="loading" style="display: block; text-align: center; padding: 32px")

        n-alert(v-else-if="error" type="error" :title="error")

        n-empty(
            v-else-if="services.length === 0"
            description="Нет сервисов с открытым доступом"
            style="padding: 32px 0"
        )

        n-list(v-else bordered)
            n-list-item(v-for="svc in services" :key="svc.id")
                n-thing
                    template(#header) {{ svc.service_name }}
                    template(#description)
                        n-text(depth="3") /{{ svc.prefix }}/
                    template(#header-extra)
                        n-button(
                            type="primary"
                            size="small"
                            @click="navigate_to(svc.prefix)"
                        ) Открыть

        n-space(style="margin-top: 16px" justify="end")
            n-button(text @click="$router.push({ name: 'login' })") ← Назад
</template>

<style scoped lang="scss">
.guest-services-page {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #18181c;
}

.services-card {
    width: 520px;
}
</style>
