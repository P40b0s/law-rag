import { computed, ref, watch } from "vue";
import useUser from "./useUser";
import { type Role } from "@/types/user_role";
import { type Permission } from '@/types/permission';


export default function useVisible()
{
    const {get_user} = useUser();
    const visible = (roles: Role[], permissions: Permission[] = []) =>
    {
        
        return computed(() =>
        {
            const user = get_user();
            if(user.value !== null)
            {
                const role_granted = roles.length == 0 ? true : roles.includes(user.value.role);
                const privilegies_granted = permissions.length == 0 ? true : permissions.some(p=>
                {
                    return user.value?.permissions.includes(p)
                })
                console.log("permissions: ", role_granted, privilegies_granted);
                return role_granted && privilegies_granted
            }
            console.log("permissions: rejected");
            return false;
        })
    }
    const disabled = (roles: Role[], privilegies: Permission[] = []) =>
    {
        return visible(roles, privilegies);
    }

    return {visible, disabled}
    
}