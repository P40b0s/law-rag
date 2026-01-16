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
import { type DepartmentsStatistic, DepartmentsStatisticSchema, EmployeesStatesCountsSchema, type EmployeesStatesCounts, } from '@/types/statistic';


class StatisticService
{
    async employees_states_counts(department_id?: string): Promise<EmployeesStatesCounts | undefined>
    {
        const path = department_id ? `statistic/employees_states_counts/${department_id}` : "statistic/employees_states_counts";
        const emp: Result<EmployeesStatesCounts, HTTPError> = await get(path, true, EmployeesStatesCountsSchema);
        if (emp.err)
        {
            notify_service.notify_error("Ошибка запроса статистики по сотрудникам", emp.err.message);
        }
        else
        {
            return emp.val;
        }
    }

    async employees_current_states(requests: {name: string, statuses: string[]}[], date?: number): Promise<DepartmentsStatistic | undefined>
    {
        const date_result = () =>
        {
            if(date)
            {
                return DateTime.parse(date).to_string(DateFormat.SerializedDate)
            }
            else return undefined;
        }
        const request = {
            requests,
            date: date_result()
        }
        const emp: Result<DepartmentsStatistic, HTTPError> = await post("statistic/employees_current_states", 'POST', true, request, DepartmentsStatisticSchema);
        if (emp.err)
        {
            notify_service.notify_error("Ошибка запроса статистики по сотрудникам", emp.err.message);
        }
        else
        {
            return emp.val;
        }
    }


    
}


export { StatisticService };