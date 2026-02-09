import { Result } from '@globalart/oxide';
import { get, post, HTTPError } from './http_client';
import { notify_service } from '../notification_service';
import { Document, DocumentArraySchema, DocumentSchema } from '@/types/document';
import { DateFormat, DateTime } from '../date';

// Типы запросов
interface DocumentRequest 
{
    sign_date: string;
    number: string;
}

interface GenerationRequest 
{
    query: string;
    limit: number;
    reranker_limit: number;
}

class DocumentsService 
{
    /**
     * Запрос документа по дате подписания и номеру
     */
    async request_document(sign_date: DateTime, number: string): Promise<Document | undefined> 
    {
        const payload: DocumentRequest = 
        {
            sign_date: sign_date.to_string(DateFormat.SerializedDate),
            number
        };

        const result: Result<Document, HTTPError> = await post(
            'documents/request_document',
            'POST',
            payload,
            DocumentSchema
        );

        if (result.isErr()) 
        {
            //notify_service.error("Ошибка запроса документа", result.unwrapErr().message);
            return undefined;
        }

        return result.unwrap();
    }

        /**
     * Запрос документа по дате подписания и номеру
     */
    async get_documents(offset: number = 0, limit: number = 30): Promise<Document[] | undefined> 
    {
        const result: Result<Document[], HTTPError> = await get(
            'documents/' + offset,
            DocumentArraySchema
        );

        if (result.isErr()) 
        {
            //notify_service.error("Ошибка запроса документа", result.unwrapErr().message);
            return undefined;
        }

        return result.unwrap();
    }

    /**
     * Создание эмбеддингов для документа по его хешу
     */
    async embedding_document(hash: string): Promise<boolean> 
    {
        const result: Result<void, HTTPError> = await get(`documents/embedding_document/${hash}`);

        if (result.isErr()) 
        {
            //notify_service.error("Ошибка создания эмбеддингов", result.unwrapErr().message);
            return false;
        }

        notify_service.success("Эмбеддинг создан", `Эмбеддинг для документа ${hash} успешно создан`);
        return true;
    }

    /**
     * Генерация ответа на основе поискового запроса
     */
    async generation_request(query: string, limit: number = 10, reranker_limit: number = 5): Promise<boolean> {
        const payload: GenerationRequest =
        {
            query,
            limit,
            reranker_limit
        };

        const result: Result<void, HTTPError> = await post(
            'documents/generation_request',
            'POST',
            payload
        );

        if (result.isErr())
        {
            //notify_service.error("Ошибка генерации ответа", result.unwrapErr().message);
            return false;
        }

        notify_service.success("Запрос обработан", "Генерация ответа выполнена успешно");
        return true;
    }

    /**
     * Удаление документа по его хешу
     */
    async delete_document(hash: string): Promise<boolean>
    {
        const result: Result<void, HTTPError> = await get(`documents/delete_document/${hash}`);

        if (result.isErr())
        {
            notify_service.error("Ошибка удаления", result.unwrapErr().message);
            return false;
        }

        notify_service.success("Документ удален", `Документ ${hash} успешно удален`);
        return true;
    }

     /**
     * Запрос документа по дате подписания и номеру
     */
    async health_check(): Promise<unknown | undefined> 
    {
        const result: Result<unknown, HTTPError> = await get(
            'health_check',
        );

        if (result.isErr()) 
        {
            //notify_service.error("Ошибка запроса документа", result.unwrapErr().message);
            return undefined;
        }

        return result.unwrap();
    }
}

export { DocumentsService };
