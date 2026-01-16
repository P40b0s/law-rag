import { useSound } from '@/composables/useSound';
import { useTheme } from '@/composables/useTheme';
import { NButton } from 'naive-ui';
import { h } from 'vue';
//import { useVtEvents, useToast } from 'vue-toastify';
import Vue3Toastify, { toast, type ToastContainerOptions, type IconProps } from 'vue3-toastify';

const {get_current_theme} = useTheme()
// useVtEvents().once('vtPaused', payload => {
//     if (payload.id === toast.id) {
//         // do something
//     }
// })
export const notify_warning = (title: string, body: string) =>
{
    const t =  toast.warning(body, 
    {
        theme: get_current_theme().value == 'dark' ? 'dark' : 'light',
    });
    console.log(t);
    //для примера можно сделать свои теплейты для уведомлений
    //const toast = useToast().authenticationError({body: "БОДИ", title: "TITLE", type: 'error'})
}
class NotificationService
{
    sound = useSound()
    warning(title: string, body?: string)
    {
        const t =  toast.warning(title, {
        theme: get_current_theme().value,
        });
        console.log(t);
        //для примера можно сделать свои теплейты для уведомлений
        //const toast = useToast().authenticationError({body: "БОДИ", title: "TITLE", type: 'error'})
    }
    error(title: string, body?: string)
    {
        //const t =  useToast().error(body, title);
        //console.log(t);
        this.sound.play_sound('error');
        const t =  toast.error(title, {
        theme: get_current_theme().value,
        });
        //для примера можно сделать свои теплейты для уведомлений
        //const toast = useToast().authenticationError({body: "БОДИ", title: "TITLE", type: 'error'})
    }
    success(title: string, body?: string)
    {
         const t =  toast.success(title, {
        theme: get_current_theme().value,
        });
        console.log(t);
        //для примера можно сделать свои теплейты для уведомлений
        //const toast = useToast().authenticationError({body: "БОДИ", title: "TITLE", type: 'error'})
    }
    test()
    {
        const t1 = h(NButton, {class: "123"}, {default:() => "123123123123"});
        //const t =  useToast().warning(t1, "BUTTON!");
        //console.log(t);
        //для примера можно сделать свои теплейты для уведомлений
        //const toast = useToast().authenticationError({body: "БОДИ", title: "TITLE", type: 'error'})
    }

    
}

const notify_service = new NotificationService();
export {notify_service}