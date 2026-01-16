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
import { AddNewEmployee, Employee, type EmployeeNewState, type Employees, EmployeeSchema, EmployeesSchema, EmployeeState, type EmployeeStates, EmployeeStateSchema, EmployeeStatesSchema, UpdateEmployee } from '@/types/employees';


class EmployeeStatesService
{
    async get_states(employee_id: string): Promise<EmployeeStates | undefined>
    {
        const emp: Result<EmployeeStates, HTTPError> = await get(`employees/states/${employee_id}`, true, EmployeeStatesSchema);
        if (emp.err)
        {
            notify_service.notify_error("Ошибка запроса списка статусов сотрудника", emp.err.message);
        }
        else
        {
            return emp.val;
        }
    }
    async add(state: EmployeeNewState): Promise<EmployeeState | undefined>
    {
        const value: Result<EmployeeState, HTTPError> = await post('employees/states/add', 'POST', true,
        {
            
            ...state,
            start_date: state.start_date?.to_string(DateFormat.SerializedDate),
            end_date: state.end_date?.to_string(DateFormat.SerializedDate),
        }, EmployeeStateSchema);
        if (value.err)
        {
            notify_service.notify_error("Ошибка добавления состояния сотрудника", value.err.message);
        }
        else
        {
            return value.val;
        }
    }

    async add_multiple(states: EmployeeNewState[]): Promise<EmployeeState[] | undefined>
    {
        const value: Result<EmployeeState[], HTTPError> = await post('employees/states/add_multiple', 'POST', true,
            states.map(s=>
            {
                return {
                ...s,
                start_date: s.start_date?.to_string(DateFormat.SerializedDate),
                end_date: s.end_date?.to_string(DateFormat.SerializedDate),
                }
            })
            , EmployeeStatesSchema);
        if (value.err)
        {
            notify_service.notify_error("Ошибка добавления состояния сотрудника", value.err.message);
        }
        else
        {
            return value.val;
        }
    }

    async edit(state: EmployeeState): Promise<EmployeeState | undefined>
    {
        const value: Result<EmployeeState, HTTPError> = await post('employees/states/edit', 'POST', true, 
        {
            ...state,
            start_date: state.start_date?.to_string(DateFormat.SerializedDate),
            end_date: state.end_date?.to_string(DateFormat.SerializedDate),
        }, EmployeeStateSchema);
        if (value.err)
        {
            notify_service.notify_error("Ошибка редактирования состояния сотрудника", value.err.message);
        }
        else
        {
            return value.val;
        }
    }
    async delete(state_id: string): Promise<boolean>
    {
        const value: Result<void, HTTPError> = await post('employees/states/delete', 'POST', true, 
        {
            id: state_id
        });
        if (value.err)
        {
            notify_service.notify_error("Ошибка удаления состояния сотрудника", value.err.message);
            return false;
        }
        else
        {
            return true;
        }
    }
}


export { EmployeeStatesService };