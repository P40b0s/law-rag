import { defineConfig } from '@rsbuild/core';
import { pluginVue } from '@rsbuild/plugin-vue';
import { pluginPug } from '@rsbuild/plugin-pug';
import { pluginSass } from '@rsbuild/plugin-sass';

const GATEWAY_PORT = process.env.PUBLIC_GATEWAY_PORT ?? '8090';
const GATEWAY_HOST = process.env.PUBLIC_GATEWAY_HOST ?? 'http://localhost';

export default defineConfig({
    plugins: [pluginVue(), pluginPug(), pluginSass()],
    source: {
        entry: { index: './src/main.ts' },
    },
    output: {
        distPath: { root: 'dist' },
        assetPrefix: '/ui/',
    },
    server: {
        port: 8091,
        host: '0.0.0.0',
        strictPort: true,
        historyApiFallback: true,
        proxy: {
            '/auth': {
                target: `${GATEWAY_HOST}:${GATEWAY_PORT}`,
                changeOrigin: true,
            },
            '/gateway-admin': {
                target: `${GATEWAY_HOST}:${GATEWAY_PORT}`,
                changeOrigin: true,
            },
        },
    },
    html: {
        template: 'index.html',
        title: 'Gateway',
        meta: {
            charset: 'UTF-8',
            viewport: 'width=device-width, initial-scale=1.0',
        },
    },
});
