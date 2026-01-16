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
import { type DictionaryWithWeight, DictionaryWithWeightSchema } from '@/types/dictionary';


class DepartmentsService
{
    async add(name: string, weight: number): Promise<DictionaryWithWeight | undefined>
    {
        const clinic: Result<DictionaryWithWeight, HTTPError> = await post('dictionaries/departments/add', 'POST', true, 
        {
            value: name,
            weight
        }, DictionaryWithWeightSchema);
        if (clinic.err)
        {
            notify_service.notify_error("Ошибка добавления отдела", clinic.err.message);
        }
        else
        {
            return clinic.val;
        }
    }

    async edit(id: string, name: string, weight: number): Promise<DictionaryWithWeight | undefined>
    {
        const clinic: Result<DictionaryWithWeight, HTTPError> = await post('dictionaries/departments/edit', 'POST', true, 
        {
            id: id,
            value: name,
            weight
        }, DictionaryWithWeightSchema);
        if (clinic.err)
        {
            notify_service.notify_error("Ошибка добавления отдела", clinic.err.message);
        }
        else
        {
            return clinic.val;
        }
    }

    async delete(id: string): Promise<boolean>
    {
        const clinic: Result<DictionaryWithWeight, HTTPError> = await post('dictionaries/departments/delete', 'POST', true, 
        {
            id: id
        });
        if (clinic.err)
        {
            notify_service.notify_error("Ошибка добавления отдела", clinic.err.message);
            return false;
        }
        else
        {
            return true;
        }
    }
   
   
    async get():  Promise<DictionaryWithWeight[]>
    {
        const r: Result<DictionaryWithWeight[], HTTPError> = await get('dictionaries/departments', true)
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


export { DepartmentsService };