import { createApp } from "vue";
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'
import App from "./App.vue";
import emitter from './services/emitter'
import sse_service from '@/services/sse'

import Vue3Toastify, { type ToastContainerOptions } from 'vue3-toastify';
import 'vue3-toastify/dist/index.css';
import '@styles/index.scss'

const app = createApp(App);
app.provide('emitter', emitter);
app.provide('sse', sse_service);
app.use(Vue3Toastify, {
    autoClose: 3000,
    position: "top-right",
    pauseOnHover: true,
    transition: 'flip',
    pauseOnFocusLoss: true,
    draggable: true,
    draggablePercent: 0.6,
    showCloseButton: true,
    hideProgressBar: false,
    closeOnClick: true,
    rtl: false
} as ToastContainerOptions);
app.mount("#app");
