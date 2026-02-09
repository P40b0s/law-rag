import { readonly, ref, type Ref } from "vue";
import { http_sevice } from "@/services/http_service/http_service";
import { Document } from "@/types/document";
import { DateTime } from "@/services/date";

const current_document: Ref<Document | null> = ref(null);
const is_loading: Ref<boolean> = ref(false);
const is_embedding: Ref<boolean> = ref(false);
const is_generating: Ref<boolean> = ref(false);

export default function useEmbeddings()
{
    const set_document = (document: Document | null) =>
    {
        current_document.value = document;
    }

    const get_document = () => readonly(current_document);
    const get_loading = () => readonly(is_loading);
    const get_embedding = () => readonly(is_embedding);
    const get_generating = () => readonly(is_generating);

    /**
     * Запрос документа по дате подписания и номеру
     */
    const request_document = async (sign_date: DateTime, number: string): Promise<boolean> =>
    {
        is_loading.value = true;
        try
        {
            const document = await http_sevice.documents_service.request_document(sign_date, number);
            if (document)
            {
                set_document(document);
                return true;
            }
            return false;
        }
        finally
        {
            is_loading.value = false;
        }
    }

    /**
     * Создание эмбеддингов для текущего документа
     */
    const embedding_current_document = async (): Promise<boolean> =>
    {
        if (!current_document.value)
        {
            return false;
        }

        is_embedding.value = true;
        try
        {
            const success = await http_sevice.documents_service.embedding_document(
                current_document.value.document_hash
            );

            if (success && current_document.value)
            {
                // Обновляем статус документа после успешного создания эмбеддингов
                current_document.value.has_embeddings = true;
                current_document.value.status = "Embedded";
            }

            return success;
        }
        finally
        {
            is_embedding.value = false;
        }
    }

    /**
     * Создание эмбеддингов для документа по хешу
     */
    const embedding_document = async (hash: string): Promise<boolean> =>
    {
        is_embedding.value = true;
        try
        {
            return await http_sevice.documents_service.embedding_document(hash);
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
        reranker_limit: number = 5
    ): Promise<boolean> =>
    {
        is_generating.value = true;
        try
        {
            return await http_sevice.documents_service.generation_request(
                query,
                limit,
                reranker_limit
            );
        }
        finally
        {
            is_generating.value = false;
        }
    }

    /**
     * Очистка текущего документа
     */
    const clear_document = () =>
    {
        current_document.value = null;
    }

    return {
        // State getters
        get_document,
        get_loading,
        get_embedding,
        get_generating,

        // Actions
        set_document,
        request_document,
        embedding_current_document,
        embedding_document,
        generation_request,
        clear_document
    }
}
