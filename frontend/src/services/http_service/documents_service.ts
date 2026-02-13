import { Result } from '@globalart/oxide';
import { get, post, HTTPError } from './http_client';
import { notify_service } from '../notification_service';
import { Document, DocumentArraySchema, DocumentSchema } from '@/types/document';
import { DateFormat, DateTime } from '../date';
import { Collection } from '@/types/collection';

// Типы запросов
interface DocumentRequest
{
    sign_date: string;
    number: string;
    collection_id: string;
}

interface EmbeddingRequest
{
    hash: string;
    collection_id: string;
}

class DocumentsService 
{
    /**
     * Запрос документа по дате подписания, номеру и коллекции
     */
    async request_document(sign_date: DateTime, number: string, collection_id: string): Promise<Document | undefined>
    {
        const payload: DocumentRequest =
        {
            sign_date: sign_date.to_string(DateFormat.SerializedDate),
            number,
            collection_id
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
     * Создание эмбеддингов для документа по его хешу и коллекции
     */
    async embedding_document(hash: string, collection_id: string): Promise<boolean>
    {
        const payload: EmbeddingRequest =
        {
            hash,
            collection_id
        };

        const result: Result<void, HTTPError> = await post(
            'documents/embedding_document',
            'POST',
            payload
        );

        if (result.isErr())
        {
            return false;
        }
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
    async get_documents_count(): Promise<number | undefined>
    {
        const result: Result<number, HTTPError> = await get(
            'documents/count',
        );

        if (result.isErr()) 
        {
            return undefined;
        }

        return result.unwrap();
    }
}

export { DocumentsService };
