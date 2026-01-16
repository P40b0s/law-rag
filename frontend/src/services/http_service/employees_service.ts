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
import { AddNewEmployee, AssotiatedEmployee, AssotiatedEmployeeSchema, CalendarArrayShema, CalendarEmployee, Employee, type Employees, EmployeeSchema, EmployeesListSchema, EmployeesSchema, UpdateEmployee } from '@/types/employees';


class EmployeesService
{
    async get_employees_with_status(department_id?: string): Promise<Employees | undefined>
    {
        const path = department_id ? `employees/${department_id}` : "employees";
        const emp: Result<Employees, HTTPError> = await get(path, true, EmployeesSchema);
        if (emp.err)
        {
            notify_service.notify_error("Ошибка запроса списка сотрудников", emp.err.message);
        }
        else
        {
            return emp.val;
        }
    }
    async add(employee: AddNewEmployee): Promise<Employee | undefined>
    {
        const value: Result<Employee, HTTPError> = await post('employees/add', 'POST', true,
        {
            
            ...employee,
            birthday: employee.birthday?.to_string(DateFormat.SerializedDate)
        }, EmployeeSchema);
        if (value.err)
        {
            notify_service.notify_error("Ошибка добавления нового сотрудника", value.err.message);
        }
        else
        {
            return value.val;
        }
    }

    async edit(employee: UpdateEmployee): Promise<Employee | undefined>
    {
        const value: Result<Employee, HTTPError> = await post('employees/edit', 'POST', true, 
        {
            
            ...employee,
            birthday: employee.birthday?.to_string(DateFormat.SerializedDate)
        }, EmployeeSchema);
        if (value.err)
        {
            notify_service.notify_error("Ошибка редактирования сотрудника", value.err.message);
        }
        else
        {
            return value.val;
        }
    }
    async delete(employee_id: string): Promise<boolean>
    {
        const value: Result<void, HTTPError> = await post('employees/delete', 'POST', true, 
        {
            id: employee_id
        });
        if (value.err)
        {
            notify_service.notify_error("Ошибка удаления сотрудника", value.err.message);
            return false;
        }
        else
        {
            return true;
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

    async get_calendar_employees(year: number, month: number, department_id?: string):  Promise<CalendarEmployee[]>
    {
        const r: Result<CalendarEmployee[], HTTPError> = await post('employees/calendar', 'POST', true, 
            {
                department_id,
                year,
                month
            },
            CalendarArrayShema
        )
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

    async assotiate_employee_with_user(user_id: string, employee_id: string): Promise<boolean>
    {
        const value: Result<void, HTTPError> = await post('employees/assotiations/add', 'POST', true, 
        {
            user_id,
            employee_id
        });
        if (value.err)
        {
            notify_service.notify_error("Ошибка сопоставления сотрудника и юзера", value.err.message);
            return false;
        }
        else
        {
            return true;
        }
    }
    async disassotiate_employee_with_user(user_id: string): Promise<boolean>
    {
        const value: Result<void, HTTPError> = await post('employees/assotiations/delete', 'POST', true, 
        {
            user_id,
        });
        if (value.err)
        {
            notify_service.notify_error("Ошибка удаления сопоставления сотрудника и юзера", value.err.message);
            return false;
        }
        else
        {
            return true;
        }
    }

    async get_assotiate_employee(user_id: string): Promise<AssotiatedEmployee | undefined>
    {
        const value: Result<AssotiatedEmployee | null, HTTPError> = await post('employees/assotiations', 'POST', true, 
        {
            user_id,
        }, AssotiatedEmployeeSchema);
        if (value.err)
        {
            notify_service.notify_error("Для текущего юзера не найден сопоставленный сотрудник", value.err.message);
        }
        return value.val ?? undefined;
        
    }

    async get_employee(employee_id: string): Promise<AssotiatedEmployee | undefined>
    {
        const value: Result<AssotiatedEmployee | null, HTTPError> = await get(`employees/id/${employee_id}`, true, AssotiatedEmployeeSchema);
        if (value.err)
        {
            notify_service.notify_error("Cотрудник не найден", value.err.message);
        }
        return value.val ?? undefined;
        
    }

    async get_employees_list(): Promise<AssotiatedEmployee[] | undefined>
    {
        const value: Result<AssotiatedEmployee[], HTTPError> = await get('employees/all', true, EmployeesListSchema);
        if (value.err)
        {
            notify_service.notify_error("Ошибка загрузки списка сотрудников", value.err.message);
        }
        return value.val ?? undefined;
        
    }
}


export { EmployeesService };