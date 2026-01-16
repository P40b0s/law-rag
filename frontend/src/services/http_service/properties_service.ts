import { createErr, type Result, createOk, type Err } from 'option-t/plain_result';
//import { notification_service } from "./notification";
import { match } from 'ts-pattern';
import useUser from '@/composables/useUser';
import {api_path, get, HTTPError, post} from './http_client';
import { UserInfo, UserLoginResponseSchema, UserInfoUpdate } from '@/types/user_info';
import { notify_service } from '../notification_service';
import { DateFormat, DateTime } from '../date';
import { DocumentSchema, DocumentsSchema, type Document, type Documents } from '@/types/document';
import { type Packet, type PacketFiles, FilesSchema, PacketSchema, type Packets,  PacketsSchema, type FilesWithDocument, FilesWithDocumentSchema } from '@/types/packet';
import { type DynamicVNode, type EditorJsModel } from '@/types/dynamic_vnode';
import { Dictionary, DictionarySchema } from '@/types/dictionary';


class PropertiesService
{
    async add(name: string): Promise<Dictionary | undefined>
    {
        const result: Result<Dictionary, HTTPError> = await post('dictionaries/properties/add', 'POST', true, 
        {
            value: name
        }, DictionarySchema);
        if (result.err)
        {
            notify_service.notify_error("Ошибка добавления свойства", result.err.message);
        }
        else
        {
            return result.val;
        }
    }

    async edit(id: string, name: string): Promise<Dictionary | undefined>
    {
        const result: Result<Dictionary, HTTPError> = await post('dictionaries/properties/edit', 'POST', true, 
        {
            id: id,
            value: name,
        }, DictionarySchema);
        if (result.err)
        {
            notify_service.notify_error("Ошибка редактирования свойтва", result.err.message);
        }
        else
        {
            return result.val;
        }
    }

    async delete(id: string): Promise<boolean>
    {
        const result: Result<Dictionary, HTTPError> = await post('dictionaries/properties/delete', 'POST', true, 
        {
            id: id
        });
        if (result.err)
        {
            notify_service.notify_error("Ошибка удаления свойства", result.err.message);
            return false;
        }
        else
        {
            return true;
        }
    }
   
   
    async get():  Promise<Dictionary[]>
    {
        const r: Result<Dictionary[], HTTPError> = await get('dictionaries/properties', true)
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
}


export { PropertiesService };