import { match } from 'ts-pattern';
import {api_path, get, HTTPError, post} from './http_client';
import { notify_service } from '../notification_service';
import { DateFormat, DateTime } from '../date';
import { DocumentSchema,  type Document} from '@/types/document';
import useModelState from '@/composables/useModelState';
import { ModelsState, ModelsStateSchema } from '@/types/models_state';
import { Result } from '@globalart/oxide';

const {set_state} = useModelState();
class ModelStateServiceService
{
    async get_state(): Promise<ModelsState | undefined>
    {
        const result: Result<ModelsState, HTTPError> = await get('models/get_models_state', ModelsStateSchema);
        if (result.isErr())
        {
            notify_service.error("Ошибка получения статуса моделей", result.unwrapErr().message);
        }
        else
        {
            const state = result.unwrap();
            set_state(state);
            return state;
        }
    }
    async load_embedding_models(): Promise<ModelsState | undefined>
    {
        const result: Result<ModelsState, HTTPError> = await get('models/load_embedding_model', ModelsStateSchema);
        if (result.isErr())
        {
            notify_service.error("Ошибка получения статуса моделей", result.unwrapErr().message);
        }
        else
        {
            const state = result.unwrap();
            set_state(state);
            return state;
        }
    }
    async unload_embedding_models(): Promise<ModelsState | undefined>
    {
        const result: Result<ModelsState, HTTPError> = await get('models/unload_embedding_model', ModelsStateSchema);
        if (result.isErr())
        {
            notify_service.error("Ошибка получения статуса моделей", result.unwrapErr().message);
        }
        else
        {
            const state = result.unwrap();
            set_state(state);
            return state;
        }
    }
    async load_generator_model(): Promise<ModelsState | undefined>
    {
        const result: Result<ModelsState, HTTPError> = await get('models/load_generator_model', ModelsStateSchema);
        if (result.isErr())
        {
            notify_service.error("Ошибка получения статуса моделей", result.unwrapErr().message);
        }
        else
        {
            const state = result.unwrap();
            set_state(state);
            return state;
        }
    }
    async unload_generator_model(): Promise<ModelsState | undefined>
    {
        const result: Result<ModelsState, HTTPError> = await get('models/unload_generator_model', ModelsStateSchema);
        if (result.isErr())
        {
            notify_service.error("Ошибка получения статуса моделей", result.unwrapErr().message);
        }
        else
        {
            const state = result.unwrap();
            set_state(state);
            return state;
        }
    }
}


export { ModelStateServiceService };