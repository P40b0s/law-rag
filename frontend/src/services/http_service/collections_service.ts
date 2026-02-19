import { Result } from '@globalart/oxide';
import { get, post, HTTPError } from './http_client';
import { notify_service } from '../notification_service';
import { Document, DocumentArraySchema, DocumentSchema } from '@/types/document';
import { DateFormat, DateTime } from '../date';
import { Collection, CollectionArraySchema, CollectionSchema } from '@/types/collection';

// Типы запросов
interface AddCollectionRequest 
{
    name: string;
    description: string;
}
interface UpdateDescriptionRequest 
{
    id: string;
    description: string;
}


class CollectionsService 
{
    /**
     * Удаление коллекции по id если в коллекции есть документы то они будут удалены вместе с коллекцией
     */
    async delete_collection(id: string): Promise<boolean> 
    {
        const result: Result<void, HTTPError> = await get(`collections/delete/${id}`);

        if (result.isErr()) 
        {
            notify_service.error("Ошибка удаления коллекции", result.unwrapErr().message);
            return false;
        }

        notify_service.success("Коллекция удалена", `Коллекция с id ${id} успешно удалена`);
        return true;
    }

    /**
     * Добавление коллекции
     */
    async add_collection(name: string, description: string): Promise<Collection | undefined>
    {
        const payload: AddCollectionRequest =
        {
            name,
            description,
        };

        const result: Result<Collection, HTTPError> = await post(
            'collections/add',
            'POST',
            payload,
            CollectionSchema
        );
        if (result.isErr())
        {
            notify_service.error("Ошибка добавления коллекции", result.unwrapErr().message);
            return undefined;
        }

        notify_service.success("Коллекция добавлена", `Коллекция ${name} успешно добавлена`);
        return result.unwrap();
    }
    /**
     * Получение всех коллекций
     */
    async get_collections(): Promise<Collection[] | undefined>
    {
        const result: Result<Collection[], HTTPError> = await get(
            'collections',
            CollectionArraySchema
        );

        if (result.isErr()) 
        {
            return undefined;
        }

        return result.unwrap();
    }
    
     /**
     * Обновление описания коллекции по id
     */
    async update_collection(id: string, description: string): Promise<boolean>
    {
        const payload: UpdateDescriptionRequest =
        {
            id,
            description,
        };

        const result: Result<void, HTTPError> = await post(
            'collections/update',
            'POST',
            payload
        );

        if (result.isErr())
        {
            notify_service.error("Ошибка обновления коллекции", result.unwrapErr().message);
            return false;
        }

        notify_service.success("Коллекция обновлена", `Коллекция успешно обновлена`);
        return true;
    }
}

export { CollectionsService };
