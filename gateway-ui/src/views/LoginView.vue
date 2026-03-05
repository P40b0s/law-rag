<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import {
    NCard, NTabs, NTabPane, NForm, NFormItem, NInput, NButton,
    NAlert, NProgress, NSpace, NDivider, NText, useMessage,
    type FormInst, type FormRules,
} from 'naive-ui'
import useAuth from '@/composables/useAuth'
import type { LoginRequest, RegisterRequest } from '@/types/auth'

const router = useRouter()
const message = useMessage()
const {
    is_authenticated, has_guest_access,
    solving, pow_iterations, error,
    login, register, get_guest_access,
} = useAuth()

// --- Redirect if already authenticated ---
if (is_authenticated.value) {
    router.push({ name: 'admin' })
}

// --- Login form ---
const login_form_ref = ref<FormInst | null>(null)
const login_loading = ref(false)
const login_data = ref<LoginRequest>({ username: '', password: '' })
const login_rules: FormRules = {
    username: [{ required: true, message: 'Введите имя пользователя', trigger: 'blur' }],
    password: [{ required: true, message: 'Введите пароль', trigger: 'blur' }],
}

const handle_login = async () => {
    await login_form_ref.value?.validate()
    login_loading.value = true
    error.value = null
    try {
        await login(login_data.value)
        message.success('Добро пожаловать!')
        router.push({ name: 'admin' })
    } catch (e: any) {
        error.value = e.message
    } finally {
        login_loading.value = false
    }
}

// --- Register form ---
const show_register = ref(false)
const reg_form_ref = ref<FormInst | null>(null)
const reg_loading = ref(false)
const reg_data = ref<RegisterRequest>({
    username: '', password: '', first_name: '', second_name: '', surname: '',
})
const reg_rules: FormRules = {
    username: [{ required: true, message: 'Обязательное поле', trigger: 'blur' }],
    password: [{ required: true, message: 'Обязательное поле', trigger: 'blur' }],
    first_name: [{ required: true, message: 'Обязательное поле', trigger: 'blur' }],
    second_name: [{ required: true, message: 'Обязательное поле', trigger: 'blur' }],
    surname: [{ required: true, message: 'Обязательное поле', trigger: 'blur' }],
}

const handle_register = async () => {
    await reg_form_ref.value?.validate()
    reg_loading.value = true
    error.value = null
    try {
        await register(reg_data.value)
        message.success('Регистрация успешна. Войдите в систему.')
        show_register.value = false
    } catch (e: any) {
        error.value = e.message
    } finally {
        reg_loading.value = false
    }
}

// --- Guest access ---
const guest_loading = ref(false)

const handle_guest_access = async () => {
    guest_loading.value = true
    error.value = null
    try {
        await get_guest_access()
        message.success('Временный доступ получен!')
        router.push({ name: 'services' })
    } catch (e: any) {
        error.value = e.message
    } finally {
        guest_loading.value = false
    }
}

const pow_progress_pct = () => {
    // difficulty=4 → ~65536 ожидаемых итераций
    return Math.min(Math.round((pow_iterations.value / 65536) * 100), 99)
}
</script>

<template lang="pug">
.login-page
    n-card.login-card(title="Gateway")
        n-tabs(type="line" animated)
            //- ── Вход ──────────────────────────────────────────────
            n-tab-pane(name="login" tab="Вход")
                n-alert(v-if="error" type="error" :title="error" style="margin-bottom: 16px")

                template(v-if="!show_register")
                    n-form(ref="login_form_ref" :model="login_data" :rules="login_rules")
                        n-form-item(label="Имя пользователя" path="username")
                            n-input(
                                v-model:value="login_data.username"
                                placeholder="username"
                                @keyup.enter="handle_login"
                            )
                        n-form-item(label="Пароль" path="password")
                            n-input(
                                v-model:value="login_data.password"
                                type="password"
                                show-password-on="click"
                                placeholder="••••••••"
                                @keyup.enter="handle_login"
                            )
                    n-space(vertical style="width: 100%")
                        n-button(
                            type="primary"
                            block
                            :loading="login_loading"
                            @click="handle_login"
                        ) Войти
                        n-button(
                            text
                            block
                            @click="show_register = true"
                        ) Нет аккаунта? Зарегистрироваться

                template(v-else)
                    n-form(ref="reg_form_ref" :model="reg_data" :rules="reg_rules")
                        n-form-item(label="Имя пользователя" path="username")
                            n-input(v-model:value="reg_data.username" placeholder="username")
                        n-form-item(label="Пароль" path="password")
                            n-input(v-model:value="reg_data.password" type="password" show-password-on="click")
                        n-form-item(label="Имя" path="first_name")
                            n-input(v-model:value="reg_data.first_name")
                        n-form-item(label="Отчество" path="second_name")
                            n-input(v-model:value="reg_data.second_name")
                        n-form-item(label="Фамилия" path="surname")
                            n-input(v-model:value="reg_data.surname")
                    n-space(vertical style="width: 100%")
                        n-button(
                            type="primary"
                            block
                            :loading="reg_loading"
                            @click="handle_register"
                        ) Зарегистрироваться
                        n-button(text block @click="show_register = false") Назад

            //- ── Гостевой доступ ───────────────────────────────────
            n-tab-pane(name="guest" tab="Гостевой доступ")
                n-alert(v-if="error" type="error" :title="error" style="margin-bottom: 16px")

                n-space(vertical align="center" style="text-align: center; padding: 8px 0")
                    n-text(depth="2")
                        | Для краткосрочного доступа без аккаунта выполните
                        | проверку «не робот» (Proof-of-Work).
                        | Браузер решит задачу автоматически — это займёт
                        | несколько секунд.

                    template(v-if="solving")
                        n-text(depth="3") Вычисляем... {{ pow_iterations.toLocaleString('ru') }} итераций
                        n-progress(
                            type="line"
                            :percentage="pow_progress_pct()"
                            :indicator-placement="'inside'"
                            style="width: 100%; margin-top: 8px"
                        )

                    n-button(
                        v-else
                        type="primary"
                        :loading="guest_loading"
                        :disabled="solving"
                        size="large"
                        @click="handle_guest_access"
                    ) {{ has_guest_access ? 'Обновить доступ' : 'Получить временный доступ' }}

                    n-text(v-if="has_guest_access" type="success" depth="2")
                        | Доступ активен.
                        a(
                            href="#"
                            style="margin-left: 4px"
                            @click.prevent="$router.push({ name: 'services' })"
                        ) Перейти к сервисам
</template>

<style scoped lang="scss">
.login-page {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #18181c;
}

.login-card {
    width: 420px;
}
</style>
