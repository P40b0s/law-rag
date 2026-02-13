import { Result } from '@globalart/oxide';
import { post, HTTPError } from './http_client';
import { notify_service } from '../notification_service';
import { QdrantContext, QdrantContextArraySchema } from '@/types/query';

// Типы запросов
interface QueryRequest {
    query: string;
    limit: number;
    reranker_limit: number;
}

interface GenerationRequest
{
    query: string;
    limit: number;
    reranker_limit: number;
}

class QueryService {
    /**
     * Поиск контекста по запросу пользователя
     * @param query - текст запроса
     * @param limit - максимальное количество результатов для поиска
     * @param reranker_limit - максимальное количество результатов после реранкинга
     * @returns массив найденных контекстов или undefined в случае ошибки
     */
    async search_results(
        query: string,
        limit: number = 10,
        reranker_limit: number = 5
    ): Promise<QdrantContext[] | undefined> {
        const payload: QueryRequest = {
            query,
            limit,
            reranker_limit
        };

        const result: Result<QdrantContext[], HTTPError> = await post(
            'query/results',
            'POST',
            payload,
            QdrantContextArraySchema
        );

        if (result.isErr()) {
            notify_service.error(
                "Ошибка поиска",
                result.unwrapErr().message
            );
            return undefined;
        }

        const results = result.unwrap();

        if (results.length === 0) {
            notify_service.info(
                "Результатов не найдено",
                "По вашему запросу ничего не найдено. Попробуйте изменить формулировку."
            );
        }

        return results;
    }

    /**
     * Генерация ответа на основе поискового запроса с указанием коллекций
     */
    async generation_request(query: string, limit: number = 10, reranker_limit: number = 5): Promise<boolean> {
        const payload: GenerationRequest =
        {
            query,
            limit,
            reranker_limit
        };

        const result: Result<void, HTTPError> = await post(
            'query/generator',
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
     * Форматирование результата для отображения
     * @param context - контекст из Qdrant
     * @returns отформатированная строка
     */
    formatContext(context: QdrantContext): string {
        return `${context.document_title} (${context.document_number} от ${context.document_sign_date}):\n${context.text}`;
    }
}

export { QueryService };
