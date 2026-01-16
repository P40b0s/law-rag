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
import { AddNewEmployee, Employee, type Employees, EmployeeSchema, EmployeesSchema, UpdateEmployee } from '@/types/employees';
import { EmployeeInformation, EmployeeInformationSchema, EmployeesInformationSchema } from '@/types/employee_information';


class InformationService
{
    async get(emplotee_id: string): Promise<EmployeeInformation[] | undefined>
    {
        const emp: Result<EmployeeInformation[], HTTPError> = await get(`employees/information/${emplotee_id}`, true, EmployeesInformationSchema);
        if (emp.err)
        {
            notify_service.notify_error("Ошибка запроса информации по сотруднику", emp.err.message);
        }
        else
        {
            return emp.val;
        }
    }
    async add(employee_id: string, property: string, value: string): Promise<EmployeeInformation | undefined>
    {
        const result: Result<EmployeeInformation, HTTPError> = await post('employees/information/add', 'POST', true,
        {
            employee_id,
            property,
            value
        }, EmployeeInformationSchema);
        if (result.err)
        {
            notify_service.notify_error("Ошибка добавления информации", result.err.message);
        }
        else
        {
            return result.val;
        }
    }

    async edit(id: string, property: string, value: string): Promise<boolean | undefined>
    {
        const result: Result<void, HTTPError> = await post('employees/information/edit', 'POST', true, 
        {
            
            id,
            property,
            value
        }, undefined);
        if (result.err)
        {
            notify_service.notify_error("Ошибка редактирования информации", result.err.message);
            return false;
        }
        else
        {
            return true;
        }
    }
    async delete(id: string): Promise<boolean>
    {
        const value: Result<void, HTTPError> = await get(`employees/delete/${id}`, true);
        if (value.err)
        {
            notify_service.notify_error("Ошибка удаления информации", value.err.message);
            return false;
        }
        else
        {
            return true;
        }
    }
    async get_properties(): Promise<string[] | undefined>
    {
        const emp: Result<string[], HTTPError> = await get(`employees/information/properties`, true, undefined);
        if (emp.err)
        {
            notify_service.notify_error("Ошибка запроса списка свойств", emp.err.message);
            return []
        }
        else
        {
            return emp.val;
        }
    }
}


export { InformationService };