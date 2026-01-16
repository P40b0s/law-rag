import { createErr, type Result, createOk, type Err } from 'option-t/plain_result';
//import { notification_service } from "./notification";
import { match } from 'ts-pattern';
import useUser from '@/composables/useUser';
import {api_path, get, HTTPError, post} from './http_client';
import { UserInfo, UserLoginResponseSchema, UserInfoUpdate } from '@/types/user_info';
import { notify_service } from '../notification_service';
import { DateFormat, DateTime } from '../date';
//import { DocumentSchema, DocumentsSchema, type Document, type Documents } from '@/types/document';
import { string } from 'zod';
import { type Packet, type PacketFiles, FilesSchema, PacketSchema, type Packets,  PacketsSchema, type FilesWithDocument, FilesWithDocumentSchema } from '@/types/packet';
import { type DynamicVNode, type EditorJsModel } from '@/types/dynamic_vnode';
import { UserService } from './user_service';
import { ClinicService } from './clinic_service';
import { DepartmentsService } from './departments_service';
import { EmployeeStatusService } from './employee_status_service';
import { EmployeesService } from './employees_service';
import { EmployeeStatesService } from './employee_states_service';
import { InformationService } from './employees_information_service';
import { PropertiesService } from './properties_service';
import { StatisticService } from './statistic_service';
import { TasksService } from './tasks_service';

class HttpService
{
    public user_service: UserService;
    public clinic_service: ClinicService;
    public department_service: DepartmentsService;
    public properties_service: PropertiesService;
    public employee_status_service: EmployeeStatusService;
    public employees_service: EmployeesService;
    public employee_states_service: EmployeeStatesService;
    public employee_information: InformationService;
    public statistic_service: StatisticService;
    public tasks_service: TasksService;
    constructor()
    {
        this.user_service = new UserService();
        this.clinic_service = new ClinicService();
        this.department_service = new DepartmentsService();
        this.employee_status_service = new EmployeeStatusService();
        this.employees_service = new EmployeesService();
        this.employee_states_service = new EmployeeStatesService();
        this.employee_information = new InformationService();
        this.properties_service = new PropertiesService();
        this.statistic_service = new StatisticService();
        this.tasks_service = new TasksService();
    }
    async login(username: string, password: string): Promise<UserInfo| undefined>
    {
        const user: Result<UserInfo, HTTPError> = await post('users/login', 'POST', false, 
        {
            username: username,
            password: password
        }, UserLoginResponseSchema);
        if (user.err)
        {
            notify_service.notify_error("Ошибка входа", user.err.message);
        }
        else
        {
            return user.val;
        }
    }
    async get_documents(date_from: DateTime, limit: number, offset: number):  Promise<Documents>
    {
        const date = date_from.to_string(DateFormat.SerializedDate);
        const req = `documents?date=${date}&limit=${limit}&offset=${offset}`
        const r: Result<Documents, HTTPError> = await get(req, false, DocumentsSchema)
        if(r.err)
        {
            console.log(r.err);
            return [];
        }
        else
        {
            return r.val;
        }
    } 
    async get_document_text(doc_id: number):  Promise<DynamicVNode|undefined>
    {
        const req = `documents/${doc_id}/text`
        const r: Result<DynamicVNode, HTTPError> = await get(req, false)
        if(r.err)
        {
            console.log(r.err);
        }
        else
        {
            return r.val;
        }
    }
    async get_document_for_editorjs(doc_id: number):  Promise<EditorJsModel|undefined>
    {
        const req = `documents/${doc_id}/editor`
        const r: Result<EditorJsModel, HTTPError> = await get(req, false)
        if(r.err)
        {
            console.log(r.err);
        }
        else
        {
            return r.val;
        }
    } 
    async get_ready_documents(date_from: DateTime, limit: number, offset: number):  Promise<Documents>
    {
        const date = date_from.to_string(DateFormat.SerializedDate);
        const req = `documents/ready?date=${date}&limit=${limit}&offset=${offset}`
        const r: Result<Documents, HTTPError> = await get(req, false, DocumentsSchema)
        if(r.err)
        {
            console.log(r.err);
            return [];
        }
        else
        {
            return r.val;
        }
    } 
    async get_ready_checked_documents(date_from: DateTime, limit: number, offset: number):  Promise<Documents>
    {
        const date = date_from.to_string(DateFormat.SerializedDate);
        const req = `documents/checked?date=${date}&limit=${limit}&offset=${offset}`
        const r: Result<Documents, HTTPError> = await get(req, false, DocumentsSchema)
        if(r.err)
        {
            console.log(r.err);
            return [];
        }
        else
        {
            return r.val;
        }
    } 
    async get_users():  Promise<UserInfo[]>
    {
        const r: Result<UserInfo[], HTTPError> = await get('users', false)
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
    async create_user_by_admin(info: FormData, username: string):  Promise<boolean>
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
    async get_packets():  Promise<Packets>
    {
        const r: Result<Packets, HTTPError> = await get('packets', true, PacketsSchema)
        if(r.err)
        {
            console.log(r.err);
            return [];
        }
        else
        {
            return r.val;
        }
    }
    async get_packet(packet_id: string):  Promise<Packet|undefined>
    {
        const r: Result<Packet, HTTPError> = await get('packets/' + packet_id, true, PacketSchema)
        if(r.err)
        {
            console.log(r.err);
        }
        else
        {
            return r.val;
        }
    }
    async get_packet_documents(packet_id: string, limit: number, offset: number):  Promise<Documents>
    {
        const req = `documents/packet?packet_id=${packet_id}&limit=${limit}&offset=${offset}`
        const r: Result<Documents, HTTPError> = await get(req, false, DocumentsSchema)
        if(r.err)
        {
            console.log(r.err);
            return [];
        }
        else
        {
            return r.val;
        }
    } 
    async  get_packet_count(packet_id: string):  Promise<number>
    {
        const req = `documents/packet/${packet_id}/count`
        const r: Result<string, HTTPError> = await get(req, false)
        if(r.err)
        {
            console.log(r.err);
            return 0;
        }
        else
        {
            return Number.parseInt(r.val) ?? 0;
        }
    } 
    async copy_files_to_complex(packet_id: string, files: string[]): Promise<boolean>
    {
        const req: Result<void, HTTPError> = await post('packets/files/copy', 'POST', true, 
        {
            packet_id: packet_id,
            files: files
        });
        if (req.err)
        {
            notify_service.notify_error("Ошибка копирования файлов", req.err.message);
            return false;
        }
        else
        {
            return true;
        }
    }
    
    async get_files(packet_id: string):  Promise<FilesWithDocument>
    {
        const r: Result<FilesWithDocument, HTTPError> = await get('packets/files/' + packet_id, true, FilesWithDocumentSchema)
        if(r.err)
        {
            console.log(r.err);
            return [];
        }
        else
        {
            return r.val;
        }
    }
    async download_file(filename: string, extension: string, hash: string, token: string, onProgress?: (loaded: number, total: number) => void) 
    {
        const xhr = new XMLHttpRequest();
        xhr.open('GET', api_path + `packets/files/download/${hash}`, true);
        xhr.responseType = 'blob';
        
        return new Promise<void>((resolve, reject) => 
        {
            xhr.onprogress = (event) => 
            {
                if (event.lengthComputable && onProgress) 
                {
                    onProgress(event.loaded, event.total);
                }
            };
          
            xhr.onload = () => 
            {
                if (xhr.status === 200) 
                {
                    const blob = xhr.response;
                    const url = URL.createObjectURL(blob);
                        const a = document.createElement('a');
                        a.href = url;
                        a.download = filename + "." + extension;
                        document.body.appendChild(a);
                        a.click();
                        // Очистка
                        window.URL.revokeObjectURL(url);
                        a.remove();
                    resolve();
                } 
                else 
                {
                    reject(new Error(`Download failed: ${xhr.statusText}`));
                }
            };
            xhr.setRequestHeader("Authorization",  "Bearer " + token);
            xhr.send();
        });
      }
    async delete_packet(packet_id: string):  Promise<void>
    {
        const r: Result<void, HTTPError> = await get('packets/delete/' + packet_id, true)
        if(r.err)
        {
            console.log(r.err);
        }
    }
    async get_file_document_mapping(hash: string):  Promise<Document|undefined>
    {
        const r: Result<Document, HTTPError> = await post('packets/document_mapping', 'POST', true, {payload: hash}, DocumentSchema)
        if(r.err)
        {
            console.log(r.err);
        }
        else
        {
            return r.val;
        }
    }
    async add_packet():  Promise<Packet|undefined>
    {
        const r: Result<Packet, HTTPError> = await get('packets/add', true, PacketSchema)
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
    
    async get_documents_count(date_from: DateTime):  Promise<number>
    {
        const date = date_from.to_string(DateFormat.SerializedDate);
        const req = `documents/count?date=${date}`
        const r: Result<string, HTTPError> = await get(req, false)
        if(r.err)
        {
            console.log(r.err);
            return 0;
        }
        else
        {
            return Number.parseInt(r.val) ?? 0;
        }
    } 
    async get_ready_documents_count(date_from: DateTime):  Promise<number>
    {
        const date = date_from.to_string(DateFormat.SerializedDate);
        const req = `documents/ready/count?date=${date}`
        const r: Result<string, HTTPError> = await get(req, false)
        if(r.err)
        {
            console.log(r.err);
            return 0;
        }
        else
        {
            return Number.parseInt(r.val) ?? 0;
        }
    } 
    async get_ready_checked_documents_count(date_from: DateTime):  Promise<number>
    {
        const date = date_from.to_string(DateFormat.SerializedDate);
        const req = `documents/checked/count?date=${date}`
        const r: Result<string, HTTPError> = await get(req, false)
        if(r.err)
        {
            console.log(r.err);
            return 0;
        }
        else
        {
            return Number.parseInt(r.val) ?? 0;
        }
    }
    async get_pdf_pages_count(doc_id: number):  Promise<number>
    {
        const req = `documents/pdf/pages/${doc_id}`
        const r: Result<string, HTTPError> = await get(req, false)
        if(r.err)
        {
            console.log(r.err);
            return 0;
        }
        else
        {
            return Number.parseInt(r.val) ?? 0;
        }
    }

    async get_pdf_images(doc_id: number, pages: number[]): Promise<Result<ReadableStreamDefaultReader<Uint8Array<ArrayBufferLike>>, HTTPError>>
    {
        const payload =  JSON.stringify({
            doc_id: doc_id,
            pages: pages
        });
        const response = await fetch(`${api_path}documents/pdf`, 
        {
            method: 'POST',
            headers: 
            {
                'Accept': 'image/webp',
                'Content-Type': 'application/json'
            },
            body: payload
        });
        
        if (!response.ok) 
        {
            return createErr(new HTTPError(`HTTP error! status: ${response.status}`));
        }
      
        if (!response.body) 
        {
            return createErr(new HTTPError('ReadableStream not supported in this browser'));
        }
        const reader = response.body.getReader();
        let image_index = 0;
        return createOk(reader);
        // while (true) 
        // {
        //     const { done, value } = await reader.read();
        //     if (done) 
        //     {
        //       console.log('Stream complete');
        //       break;
        //     }
      
        //     // Создаем Blob из полученных данных
        //     const blob = new Blob([value], { type: 'image/webp' });
        //     const imageUrl = URL.createObjectURL(blob);
        //   }
    }
}

const http_sevice = new HttpService();
export { http_sevice };