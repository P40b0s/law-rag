import { http_sevice } from '@/services/http_service/http_service';
import { ref, readonly } from 'vue';
export const avatars = ref<Map<string, string|null>>(new Map<string, string|null>());

const create_avatar_url = async (user_id: string, data: Uint8Array|undefined): Promise<string|null> => 
{
    if(data)
    {
        const type = detectImageType(data) || 'image/png' // По умолчанию PNG
        //Uint8Array может быть создан на основе разных типов буферов (ArrayBuffer или SharedArrayBuffer), но Blob ожидает только ArrayBuffer.
        const safeData = new Uint8Array(data.slice());
        const blob = new Blob([safeData], {type})
        const url = URL.createObjectURL(blob)
        avatars.value.set(user_id, url);
        return url
    }
    return null;
}
const create_avatar_url_from_blob = async (user_id: string, blob: Blob): Promise<string|null> => 
{
    const url = URL.createObjectURL(blob)
    avatars.value.set(user_id, url);
    return url
}
// Функция определения типа изображения по первым байтам
const detectImageType = (data: Uint8Array|undefined): string | null => 
{
    if(data)
    {
        if (data.length < 4) return null
    
        // PNG: \x89PNG
        if (data[0] === 0x89 && data[1] === 0x50 && data[2] === 0x4E && data[3] === 0x47) {
            return 'image/png'
        }
        
        // JPEG: \xFF\xD8\xFF
        if (data[0] === 0xFF && data[1] === 0xD8 && data[2] === 0xFF) {
            return 'image/jpeg'
        }
        
        // WebP: RIFF....WEBP
        if (data.length > 12 &&
            data[0] === 0x52 && data[1] === 0x49 && data[2] === 0x46 && data[3] === 0x46 &&
            data[8] === 0x57 && data[9] === 0x45 && data[10] === 0x42 && data[11] === 0x50) {
            return 'image/webp'
        }
    }
    return null
}
export const useImage = () =>
{
    const get_avatar = async (user_id: string) =>
    {
        const ava = avatars.value.get(user_id);
        if(ava === null || typeof ava === 'string' || user_id.length < 20)
        {
            return ava
        }
        else
        {
            const avatar = await http_sevice.get_avatar(user_id);
            return await create_avatar_url(user_id, avatar)
        }
    }
    const update_avatar = async (user_id: string, avatar: Uint8Array) =>
    {
        const ava = avatars.value.get(user_id);
        if(typeof ava === 'string')
        {
            URL.revokeObjectURL(ava);
            let new_uri = await create_avatar_url(user_id, avatar);
            avatars.value.set(user_id, new_uri);
        }
    }
    /**
     * обновляем изображение аватара в локальном хранилище
     */
    const update_avatar_from_blob = async (user_id: string, avatar: Blob) =>
    {
        const ava = avatars.value.get(user_id);
        if(typeof ava === 'string')
        {
            URL.revokeObjectURL(ava);
            let new_uri = await create_avatar_url_from_blob(user_id, avatar);
            avatars.value.set(user_id, new_uri);
        }
    }
    const get_avatars = () =>
    {
        return readonly(avatars)
    }
    return {get_avatar, get_avatars, update_avatar, update_avatar_from_blob}
}