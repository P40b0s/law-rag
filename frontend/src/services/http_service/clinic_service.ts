import { createErr, type Result, createOk, type Err } from 'option-t/plain_result';
//import { notification_service } from "./notification";
import { match } from 'ts-pattern';
import useUser from '@/composables/useUser';
import {api_path, get, HTTPError, post} from './http_client';
import { UserInfo, UserLoginResponseSchema, UserInfoUpdate } from '@/types/user_info';
import { notify_service } from '../notification_service';
import { DateFormat, DateTime } from '../date';
import { DocumentSchema, DocumentsSchema, type Document, type Documents } from '@/types/document';
import { string } from 'zod';
import { type Packet, type PacketFiles, FilesSchema, PacketSchema, type Packets,  PacketsSchema, type FilesWithDocument, FilesWithDocumentSchema } from '@/types/packet';
import { type DynamicVNode, type EditorJsModel } from '@/types/dynamic_vnode';
import { ClinicDictionary, ClinicDictionarySchema } from '@/types/clinic_dictionary';


class ClinicService
{
    async add(name: string, addresse: string): Promise<ClinicDictionary| undefined>
    {
        const clinic: Result<ClinicDictionary, HTTPError> = await post('dictionaries/clinics/add', 'POST', true, 
        {
            name: name,
            addresse: addresse
        }, ClinicDictionarySchema);
        if (clinic.err)
        {
            notify_service.notify_error("Ошибка добавления поликлинники", clinic.err.message);
        }
        else
        {
            return clinic.val;
        }
    }

    async edit(id: string, name: string, addresse: string): Promise<ClinicDictionary| undefined>
    {
        const clinic: Result<ClinicDictionary, HTTPError> = await post('dictionaries/clinics/edit', 'POST', true, 
        {
            id: id,
            name: name,
            addresse: addresse
        }, ClinicDictionarySchema);
        if (clinic.err)
        {
            notify_service.notify_error("Ошибка добавления поликлинники", clinic.err.message);
        }
        else
        {
            return clinic.val;
        }
    }

    async delete(id: string): Promise<void>
    {
        const clinic: Result<ClinicDictionary, HTTPError> = await post('dictionaries/clinics/delete', 'POST', true, 
        {
            id: id
        }, undefined);
        if (clinic.err)
        {
            notify_service.notify_error("Ошибка добавления поликлинники", clinic.err.message);
        }
    }
   
   
    async get_clinics():  Promise<ClinicDictionary[]>
    {
        const r: Result<ClinicDictionary[], HTTPError> = await get('dictionaries/clinics', true)
        if(r.err)
        {
            console.log(r.err);
            return [];
        }
        else
        {
            //console.log("Получены юзеры", r.val);
            return r.val;
        }
    } 

    async delete_user(id: string):  Promise<void>
    {
        const r: Result<FilesWithDocument, HTTPError> = await get('users/delete/' + id, true)
        if(r.err)
        {
            console.log(r.err); 
        }
    }
    async get_user(id: string):  Promise<UserInfo|undefined>
    {
        const r: Result<UserInfo, HTTPError> = await get('users/'+ id, true)
        if(r.err)
        {
            console.log(r.err);
            return undefined;
        }
        else
        {
            return r.val;
        }
    } 
    async get_avatar(id: string):  Promise<Uint8Array|undefined>
    {
        const r: Result<Uint8Array|undefined, HTTPError> = await get('users/avatar/'+ id, true)
        if(r.err)
        {
            console.log(r.err);
            return undefined;
        }
        else
        {
            if(r.val)
            {
                return new Uint8Array(r.val);
            }
            else
                return undefined;
        }
    } 

    async get_profile():  Promise<UserInfo|undefined>
    {
        const r: Result<UserInfo, HTTPError> = await get('users/profile', true)
        if(r.err)
        {
            console.log(r.err);
            notify_service.notify_error("Ошибка получения информации о профиле", r.err.message);
            return undefined;
        }
        else
        {
            return r.val;
        }
    } 
    async profile_update(info: FormData, username: string):  Promise<void>
    {
        const r: Result<void, HTTPError> = await post('users/update', 'POST', true, info)
        if(r.err)
        {
            console.log(r.err);
            notify_service.notify_error("Ошибка обновления данных профиля", r.err.message);
            return undefined;
        }
        else
        {
            notify_service.notify_success( "Обновление успешно", `Данные профиля ${username} успешно обновлены`);
            return r.val;
        }
    } 
    async admin_profile_update(info: FormData, username: string):  Promise<void>
    {
        const r: Result<void, HTTPError> = await post('users/update_by_admin', 'POST', true, info)
        if(r.err)
        {
            console.log(r.err);
            notify_service.notify_error("Ошибка обновления данных профиля", r.err.message);
            return undefined;
        }
        else
        {
            notify_service.notify_success( "Обновление успешно", `Данные профиля ${username} успешно обновлены`);
            return r.val;
        }
    } 
    async create_user(info: FormData, username: string):  Promise<boolean>
    {
        const r: Result<void, HTTPError> = await post('users/create', 'POST', true, info)
        if(r.err)
        {
            console.log(r.err);
            notify_service.notify_error("Ошибка создания профиля", r.err.message);
            return false;
        }
        else
        {
            notify_service.notify_success( "Обновление успешно", `Профиль ${username} успешно создан`);
            return true;
        }
    }
    public async create_user_by_admin(info: FormData, username: string):  Promise<boolean>
    {
        const r: Result<void, HTTPError> = await post('users/create_user_by_admin', 'POST', true, info)
        if(r.err)
        {
            console.log(r.err);
            notify_service.notify_error("Ошибка создания профиля", r.err.message);
            return false;
        }
        else
        {
            notify_service.notify_success( "Обновление успешно", `Профиль ${username} успешно создан`);
            return true;
        }
    }
}


export { ClinicService };