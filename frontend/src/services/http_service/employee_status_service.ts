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

import { EmployeeStatus, EmployeeStatusSchema } from '@/types/employee_status';


class EmployeeStatusService
{
    async add(status: string, description: string, is_disease: boolean, tracing: boolean, on_work_place: boolean, color: string, logo?: string): Promise<EmployeeStatus| undefined>
    {
        const clinic: Result<EmployeeStatus, HTTPError> = await post('dictionaries/employee_statuses/add', 'POST', true, 
        {
            status: status,
            description: description,
            is_disease: is_disease,
            tracing: tracing,
            on_work_place: on_work_place,
            color,
            logo
        }, EmployeeStatusSchema);
        if (clinic.err)
        {
            notify_service.notify_error("Ошибка добавления статуса содрудников", clinic.err.message);
        }
        else
        {
            return clinic.val;
        }
    }

    async edit(id: string, status: string, description: string, is_disease: boolean, tracing: boolean, on_work_place: boolean, color: string, logo?: string): Promise<EmployeeStatus| undefined>
    {
        const clinic: Result<EmployeeStatus, HTTPError> = await post('dictionaries/employee_statuses/edit', 'POST', true, 
        {
            id: id,
            status: status,
            description: description,
            is_disease: is_disease,
            tracing: tracing,
            on_work_place: on_work_place,
            color,
            logo
        }, EmployeeStatusSchema);
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
        const clinic: Result<void, HTTPError> = await post('dictionaries/employee_statuses/delete', 'POST', true, 
        {
            id: id
        });
        if (clinic.err)
        {
            notify_service.notify_error("Ошибка добавления поликлинники", clinic.err.message);
        }
    }
   
   
    async get():  Promise<EmployeeStatus[]>
    {
        const r: Result<EmployeeStatus[], HTTPError> = await get('dictionaries/employee_statuses', true)
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


export { EmployeeStatusService };