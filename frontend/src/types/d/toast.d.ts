import type { ToastPluginAPI, CustomMethods, ToastOptions } from 'vue-toastify';

declare module 'vue-toastify' 
{
    type NotifyType = "error" | "info" | "warning" | "success" | undefined;
    interface MyMethods extends CustomMethods 
    {
        authenticationError(options: ToastOptions);
    }

    function useToast(): ToastPluginAPI & MyMethods;
}