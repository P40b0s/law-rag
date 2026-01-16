import { ref } from "vue";
import useUser from "./useUser";
import { type TaskFile } from "@/types/task";
import { api_path } from "@/services/http_service/http_client";

interface UploadOptions {
    on_progress_update?: (percent: number) => void;
    on_complete?: () => void;
    on_error?: (error: string) => void;
}

export const useFileUpload = () =>
{
    const {get_token} = useUser();
    const percentage = ref(0);
    const completed = ref(false);
    const error = ref<string>();
    const upload_file_async = (path: string, file: File, options: UploadOptions = {}): Promise<void> =>
    {
        return new Promise((resolve, reject) => {
            const formData = new FormData()
            formData.append('file', file);
            const xhr = new XMLHttpRequest();

            xhr.upload.addEventListener('progress', (e) => 
            {
                if (e.lengthComputable) 
                {
                    
                    percentage.value = Math.round((e.loaded / e.total) * 100);
                    console.log(`Процесс загрузки файла ${percentage.value}`);
                    if(options.on_progress_update)
                        options.on_progress_update(percentage.value);
                }
            });

            xhr.addEventListener('loadend', () => 
            {
                if (xhr.status >= 200 && xhr.status < 300) 
                {
                    console.log(`Файл ${file.name} успешно загружен`);
                    completed.value = true;
                    if(options.on_complete)
                        options.on_complete()
                    resolve();
                } 
                else 
                {
                    console.log(`Ошибка загрузки файла ${file.name} ${ xhr.statusText || 'Unknown error'}`);
                    error.value = xhr.statusText || 'Unknown error';
                    if(options.on_error)
                        options.on_error(error.value);
                    reject(new Error(error.value));
                }
            });
            xhr.addEventListener('error', () => 
            {
                error.value = 'Network error';
                if(options.on_error)
                    options.on_error(error.value);
                reject(new Error(error.value));
            });

            // Обработка таймаута
            xhr.addEventListener('timeout', () => 
            {
                error.value = 'Request timeout';
                if(options.on_error)
                    options.on_error(error.value);
                reject(new Error(error.value));
            });
            xhr.open('POST', api_path + path , true);
            xhr.timeout = 300000; //5 мин
            xhr.setRequestHeader("Authorization",  "Bearer " + get_token());
            xhr.send(formData);
        })
    }

    const download_file_async = (file_id: string, file_name: string,  path: string, options: UploadOptions = {}) => 
    {
        const xhr = new XMLHttpRequest();
        xhr.open('GET', api_path + path , true);
        xhr.responseType = 'blob';
        
        return new Promise<void>((resolve, reject) => 
        {
            xhr.onprogress = (e) => 
            {
                if (e.lengthComputable) 
                {
                    percentage.value = Math.round((e.loaded / e.total) * 100);
                    console.log(`Процесс загрузки файла ${percentage.value}`);
                    if(options.on_progress_update)
                        options.on_progress_update(percentage.value);
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
                        a.download = file_name;
                        document.body.appendChild(a);
                        a.click();
                        // Очистка
                        window.URL.revokeObjectURL(url);
                        a.remove();
                        if(options.on_complete)
                            options.on_complete()
                    resolve();
                } 
                else 
                {
                    error.value = `Download failed: ${xhr.statusText}`;
                    if(options.on_error)
                        options.on_error(error.value);
                    reject(new Error(error.value));
                }
            };
            xhr.setRequestHeader("Authorization",  "Bearer " + get_token());
            xhr.send();
        });
      }

    return {completed, percentage, error, upload_file_async, download_file_async}
}