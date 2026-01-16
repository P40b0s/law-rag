import router from "@/router";
import emitter from "@/services/emitter";
import { base64_to_uint8_array, load_from_localstorage, remove_from_localstorage, save_to_localstorage } from "@/services/helpers";
import { http_sevice } from "@/services/http_service/http_service";
import { notify_service } from "@/services/notification_service";
import { type AssotiatedEmployee } from "@/types/employees";
import { Permission } from "@/types/permission";
import { UserInfo } from "@/types/user_info";
import { Role } from "@/types/user_role";
import { match } from "ts-pattern";
import { computed, readonly, ref, Ref, watch, type WatchStopHandle } from "vue";
import { useDictionaries } from "./useDictionaries";

const {load_dictionaries_data} = useDictionaries();
const user: Ref<UserInfo| null> = ref(null);

export const is_authentificated = computed(()=> !!user.value && !!user.value.token)
export default function useUser()
{
    const set_user = async (new_user: UserInfo) => 
    {
        save_to_localstorage('user', new_user);
        user.value = new_user;
        await load_dictionaries_data();
    }

    const load_user = async (): Promise<boolean> => 
    {
        const u = load_from_localstorage<UserInfo>('user')
        if(u)
        {
            user.value = u;
            if (user.value)
            {
                await load_dictionaries_data();
                return true;
            }
                
            else
                return false;
        }
        else
        {
            return false;
        }
    }
    const get_role = (): Role => 
    {
        return user.value?.role ?? 'Undefined'
    }
    const get_role_name = (r: Role) =>
    {
        
        return match(r)
        .with('Administrator', () => "Администратор")
        .with('User', () => "Юзер")
        .otherwise(() => "Неизвестно")
    }
    const get_permission_description = (p: Permission) =>
    {
        return match(p)
        .with('All', () => "Все права доступа")
        .with('Duty', () => "Редактирование статусов и состава работников")
        .with('Boss', ()=> "Редактирование статусов и состава работников оносящихся к текущему пользователю")
        .with('View', () => "Только просмотр")
        .exhaustive()
    }
    const get_permissions = (): Permission[] => 
    {
        return user.value?.permissions ?? [];
    }
    const exit = () => 
    {
        remove_from_localstorage('user')
        //localStorage.removeItem("user");
        user.value = null;
        router.push({name: 'login'});
    }
    const get_user = () => readonly(user);
    const get_token = (): string | undefined | null => 
    {
        return user.value?.token;
    }
    const remove_token = () => 
    {
        if(user.value)
        {
            user.value.token = undefined;
            save_to_localstorage('user', user.value);
        }
    }
    
    return {set_user, load_user, get_role, get_permissions, exit, get_user, get_token, get_permission_description, get_role_name, remove_token}
}