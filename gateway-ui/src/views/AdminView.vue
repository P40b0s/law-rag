<script setup lang="ts">
import { h, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import {
    NButton, NCard, NCheckbox, NDataTable, NDivider, NForm, NFormItem,
    NIcon, NInput, NModal, NPopconfirm, NSpace, NTag, NText, NTooltip,
    useMessage, type DataTableColumns, type FormInst, type FormRules,
} from 'naive-ui'
import { AddCircleOutline, CreateOutline, TrashOutline, LogOutOutline } from '@vicons/ionicons5'
import { gateway_service } from '@/services/gateway_service'
import useAuth from '@/composables/useAuth'
import type { ServiceRequest, ServiceResponse } from '@/types/service'

const router = useRouter()
const message = useMessage()
const { is_authenticated, logout } = useAuth()

if (!is_authenticated.value) {
    router.push({ name: 'login' })
}

// --- State ---
const services = ref<ServiceResponse[]>([])
const loading = ref(false)
const modal_open = ref(false)
const editing_id = ref<number | null>(null)
const form_ref = ref<FormInst | null>(null)
const form_loading = ref(false)

const empty_form = (): ServiceRequest => ({
    prefix: '',
    service_name: '',
    targets: [],
    permissions: [],
    allow_guests: false,
})

const form = ref<ServiceRequest>(empty_form())

// raw string inputs for array fields
const targets_raw = ref('')
const permissions_raw = ref('')

const form_rules: FormRules = {
    prefix: [{ required: true, message: 'Обязательное поле', trigger: 'blur' }],
    service_name: [{ required: true, message: 'Обязательное поле', trigger: 'blur' }],
}

// --- Load ---
const load_services = async () => {
    loading.value = true
    try {
        services.value = await gateway_service.list()
    } catch (e: any) {
        message.error(e.message)
    } finally {
        loading.value = false
    }
}

onMounted(load_services)

// --- Modal helpers ---
const open_create = () => {
    editing_id.value = null
    form.value = empty_form()
    targets_raw.value = ''
    permissions_raw.value = ''
    modal_open.value = true
}

const open_edit = (row: ServiceResponse) => {
    editing_id.value = row.id
    form.value = {
        prefix: row.prefix,
        service_name: row.service_name,
        targets: [...row.targets],
        permissions: [...row.permissions],
        allow_guests: row.allow_guests,
    }
    targets_raw.value = row.targets.join('\n')
    permissions_raw.value = row.permissions.join('\n')
    modal_open.value = true
}

const parse_lines = (raw: string): string[] =>
    raw.split('\n').map(s => s.trim()).filter(s => s.length > 0)

const handle_save = async () => {
    await form_ref.value?.validate()
    form.value.targets = parse_lines(targets_raw.value)
    form.value.permissions = parse_lines(permissions_raw.value)

    form_loading.value = true
    try {
        if (editing_id.value === null) {
            const created = await gateway_service.create(form.value)
            services.value.push(created)
            message.success('Сервис создан')
        } else {
            const updated = await gateway_service.update(editing_id.value, form.value)
            const idx = services.value.findIndex(s => s.id === editing_id.value)
            if (idx !== -1) services.value[idx] = updated
            message.success('Сервис обновлён')
        }
        modal_open.value = false
    } catch (e: any) {
        message.error(e.message)
    } finally {
        form_loading.value = false
    }
}

const handle_delete = async (id: number) => {
    try {
        await gateway_service.delete(id)
        services.value = services.value.filter(s => s.id !== id)
        message.success('Сервис удалён')
    } catch (e: any) {
        message.error(e.message)
    }
}

const handle_logout = () => {
    logout()
    router.push({ name: 'login' })
}

// --- Table columns ---
const columns: DataTableColumns<ServiceResponse> = [
    { title: 'Префикс', key: 'prefix', width: 120 },
    { title: 'Имя сервиса', key: 'service_name', width: 180 },
    {
        title: 'Адреса (targets)',
        key: 'targets',
        render: (row) => h(NSpace, { wrap: true }, () =>
            row.targets.map(t => h(NTag, { size: 'small', bordered: false }, () => t))
        ),
    },
    {
        title: 'Права',
        key: 'permissions',
        width: 160,
        render: (row) => h(NSpace, { wrap: true }, () =>
            row.permissions.map(p => h(NTag, { size: 'small', type: 'info', bordered: false }, () => p))
        ),
    },
    {
        title: 'Гости',
        key: 'allow_guests',
        width: 80,
        render: (row) => h(
            NTag,
            { type: row.allow_guests ? 'success' : 'default', size: 'small', bordered: false },
            () => row.allow_guests ? 'Да' : 'Нет',
        ),
    },
    {
        title: '',
        key: 'actions',
        width: 100,
        render: (row) => h(NSpace, {}, () => [
            h(NTooltip, {}, {
                trigger: () => h(NButton, {
                    size: 'small', quaternary: true, circle: true,
                    onClick: () => open_edit(row),
                }, { icon: () => h(NIcon, {}, () => h(CreateOutline)) }),
                default: () => 'Редактировать',
            }),
            h(NPopconfirm, {
                onPositiveClick: () => handle_delete(row.id),
            }, {
                trigger: () => h(NButton, {
                    size: 'small', quaternary: true, circle: true, type: 'error',
                }, { icon: () => h(NIcon, {}, () => h(TrashOutline)) }),
                default: () => `Удалить «${row.service_name}»?`,
            }),
        ]),
    },
]
</script>

<template lang="pug">
.admin-page
    n-card
        template(#header)
            n-space(justify="space-between" align="center" style="width: 100%")
                n-text(strong style="font-size: 18px") Управление сервисами
                n-space
                    n-button(type="primary" @click="open_create")
                        template(#icon)
                            n-icon: add-circle-outline
                        | Добавить
                    n-button(quaternary circle @click="handle_logout")
                        template(#icon)
                            n-icon: log-out-outline

        n-data-table(
            :columns="columns"
            :data="services"
            :loading="loading"
            :bordered="false"
            size="small"
        )

//- ── Модальное окно создания/редактирования ──────────────────────────
n-modal(
    v-model:show="modal_open"
    preset="card"
    :title="editing_id === null ? 'Новый сервис' : 'Редактировать сервис'"
    style="width: 520px"
    :mask-closable="false"
)
    n-form(ref="form_ref" :model="form" :rules="form_rules" label-placement="top")
        n-form-item(label="Префикс (часть URL)" path="prefix")
            n-input(
                v-model:value="form.prefix"
                placeholder="rag"
                :disabled="editing_id !== null"
            )
        n-form-item(label="Название сервиса" path="service_name")
            n-input(v-model:value="form.service_name" placeholder="rag-service")
        n-form-item(label="Адреса бэкенда (targets, по одному на строку)")
            n-input(
                v-model:value="targets_raw"
                type="textarea"
                :rows="3"
                placeholder="http://localhost:8081\nhttp://localhost:8082"
            )
        n-form-item(label="Требуемые права (permissions, по одному на строку)")
            n-input(
                v-model:value="permissions_raw"
                type="textarea"
                :rows="2"
                placeholder="read\nwrite"
            )
        n-form-item(label="Разрешить гостевой доступ")
            n-checkbox(v-model:checked="form.allow_guests") Разрешить

    template(#footer)
        n-space(justify="end")
            n-button(@click="modal_open = false") Отмена
            n-button(type="primary" :loading="form_loading" @click="handle_save") Сохранить
</template>

<style scoped lang="scss">
.admin-page {
    max-width: 1100px;
    margin: 32px auto;
    padding: 0 16px;
}
</style>
