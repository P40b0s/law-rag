import { createErr, type Result, createOk, type Err } from 'option-t/plain_result';
//import { notification_service } from "./notification";
import { match } from 'ts-pattern';
import useUser from '@/composables/useUser';
import {api_path, get, HTTPError, post} from './http_client';
import { UserInfo, UserLoginResponseSchema, UserInfoUpdate } from '@/types/user_info';
import { notify_service } from '../notification_service';
import { DateFormat, DateTime, tuple_to_rfc3339 } from '../date';
import { DocumentSchema, DocumentsSchema, type Document, type Documents } from '@/types/document';
import { string } from 'zod';
import { type Packet, type PacketFiles, FilesSchema, PacketSchema, type Packets,  PacketsSchema, type FilesWithDocument, FilesWithDocumentSchema } from '@/types/packet';
import { type DynamicVNode, type EditorJsModel } from '@/types/dynamic_vnode';
import { Dictionary, DictionarySchema } from '@/types/dictionary';
import { TagsResponse, TagsResponseSchema, TaskFile, TaskFilesSchema, TaskFilter, TaskSchema, TasksSchema, type Task } from '@/types/task';


class TasksService
{
    async add(task: Task): Promise<void | undefined>
    {
        const result: Result<void, HTTPError> = await post('tasks/add',
        'POST', 
        true, 
        {
            ...task,
            target_date: task.target_date?.to_string(DateFormat.SerializedDate),
            added_by: [task.added_by[0], task.added_by[1]?.to_string(DateFormat.SerializedDateTime)],
            changed_by: task.changed_by ? [task.changed_by[0], task.changed_by[1]?.to_string(DateFormat.SerializedDateTime)] : undefined,
        },
        TaskSchema);
        if (result.err)
        {
            notify_service.notify_error("Ошибка добавления задачи", result.err.message);
        }
        else
        {
            return result.val;
        }
    }

    async edit(task: Task): Promise<void | undefined>
    {
        const result: Result<void, HTTPError> = await post('tasks/edit',
        'POST', 
        true, 
        {
            ...task,
            target_date: task.target_date?.to_string(DateFormat.SerializedDate),
            added_by: [task.added_by[0], task.added_by[1]?.to_string(DateFormat.SerializedDateTime)],
            changed_by: task.changed_by ? [task.changed_by[0], task.changed_by[1]?.to_string(DateFormat.SerializedDateTime)] : undefined,
        },
        TaskSchema);
        if (result.err)
        {
            notify_service.notify_error("Ошибка редактирования задачи", result.err.message);
        }
        else
        {
            return result.val;
        }
    }

    async delete(id: string): Promise<boolean>
    {
        const result: Result<void, HTTPError> = await post('tasks/delete', 'POST', true, 
        {
            id: id
        });
        if (result.err)
        {
            notify_service.notify_error("Ошибка удаления задачи", result.err.message);
            return false;
        }
        else
        {
            return true;
        }
    }
    async delete_file(id: string): Promise<boolean>
    {
        const result: Result<void, HTTPError> = await post('tasks/delete_file', 'POST', true, 
        {
            id: id
        });
        if (result.err)
        {
            notify_service.notify_error("Ошибка удаления задачи", result.err.message);
            return false;
        }
        else
        {
            return true;
        }
    }
    async get(department_id?: string):  Promise<Task[]>
    {
        const path = department_id ? `tasks/${department_id}` : 'tasks';
        const r: Result<Task[], HTTPError> = await get(path, true, TasksSchema)
        if(r.err)
        {
            console.log(r.err);
            notify_service.notify_error("Ошибка получения списка задач", r.err.message);
            return [];
        }
        else
        {
            return r.val;
        }
    } 

    async get_filtered(filter: TaskFilter):  Promise<Task[]>
    {
        const path = `tasks/filter`;
        const status = (!filter.status || filter.status.length == 0) ? undefined : filter.status;
        const r: Result<Task[], HTTPError> = await post(path, 'POST', true, 
        {
            ...filter,
            dates_range: filter.dates_range ? tuple_to_rfc3339(filter.dates_range) : undefined,
            status: status,
        } as TaskFilter, TasksSchema)
        if(r.err)
        {
            console.log(r.err);
            notify_service.notify_error("Ошибка получения списка задач", r.err.message);
            return [];
        }
        else
        {
            return r.val;
        }
    } 
    async get_tags():  Promise<string[]>
    {
        const r: Result<TagsResponse, HTTPError> = await get("tasks/tags", true, TagsResponseSchema)
        if(r.err)
        {
            console.log(r.err);
            notify_service.notify_error("Ошибка получения списка тегов", r.err.message);
            return [];
        }
        else
        {
            return r.val.tags;
        }
    } 
    async get_files(task_id: string):  Promise<TaskFile[]>
    {
        const path = `tasks/${task_id}/files`;
        const r: Result<TaskFile[], HTTPError> = await get(path, true, TaskFilesSchema)
        if(r.err)
        {
            console.log(r.err);
            notify_service.notify_error("Ошибка получения списка задач", r.err.message);
            return [];
        }
        else
        {
            return r.val;
        }
    } 
}


export { TasksService };