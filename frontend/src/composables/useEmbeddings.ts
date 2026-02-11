import { readonly, ref, type Ref } from "vue";
import { http_service } from "@/services/http_service/http_service";
import { Document } from "@/types/document";
import { DateTime } from "@/services/date";

const is_loading: Ref<boolean> = ref(false);
const is_embedding: Ref<boolean> = ref(false);
const is_generating: Ref<boolean> = ref(false);

export default function useEmbeddings()
{
    const get_loading = () => readonly(is_loading);
    const get_embedding = () => readonly(is_embedding);
    const get_generating = () => readonly(is_generating);

    /**
     * Запрос документа по дате подписания и номеру
     */
    const request_document = async (sign_date: DateTime, number: string, collection_id: string): Promise<Document| undefined> =>
    {
        is_loading.value = true;
        try
        {
            const document = await http_service.documents_service.request_document(sign_date, number, collection_id);
            return document;
        }
        finally
        {
            is_loading.value = false;
        }
    }

    /**
     * Создание эмбеддингов для документа по хешу
     */
    const embedding_document = async (hash: string, collection_id: string): Promise<boolean> =>
    {
        is_embedding.value = true;
        try
        {
            return await http_service.documents_service.embedding_document(hash, collection_id);
        }
        finally
        {
            is_embedding.value = false;
        }
    }

    /**
     * Генерация ответа на основе поискового запроса
     */
    const generation_request = async (
        query: string,
        limit: number = 10,
        reranker_limit: number = 5,
       
    ): Promise<boolean> =>
    {
        is_generating.value = true;
        try
        {
            return await http_service.documents_service.generation_request(
                query,
                limit,
                reranker_limit,
            );
        }
        finally
        {
            is_generating.value = false;
        }
    }
    return {
        // State getters
        get_loading,
        get_embedding,
        get_generating,

        // Actions
        request_document,
        embedding_document,
        generation_request,
    }
}
