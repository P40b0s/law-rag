import { ref, computed } from 'vue'
import { http_service } from '@/services/http_service/http_service'
import type { Document, DocumentStatus } from '@/types/document'

// Глобальное состояние документов
const documents = ref<Document[]>([])
const currentDocument = ref<Document | null>(null)
const totalCount = ref(0)
const loading = ref(false)

export default function useDocuments()
{
  const loadDocuments = async (offset: number = 0, limit: number = 30) =>
  {
    loading.value = true
    try
    {
      const [result, count] = await Promise.all([
        http_service.documents_service.get_documents(offset, limit),
        http_service.documents_service.get_documents_count()
      ])
      if (result) documents.value = result
      totalCount.value = count ?? 0
    }
    finally
    {
      loading.value = false
    }
  }

  const deleteDocument = async (hash: string): Promise<boolean> =>
  {
    const success = await http_service.documents_service.delete_document(hash)
    if (success)
    {
      const index = documents.value.findIndex(d => d.document_hash === hash)
      if (index !== -1)
      {
        documents.value.splice(index, 1)
        totalCount.value--
      }
      if (currentDocument.value?.document_hash === hash)
      {
        currentDocument.value = null
      }
    }
    return success
  }

  /**
   * Добавляет новый документ или обновляет существующий в списке.
   * Также устанавливает его как текущий документ.
   */
  const addOrUpdateDocument = (doc: Document) =>
  {
    const index = documents.value.findIndex(d => d.document_hash === doc.document_hash)
    if (index !== -1)
    {
      documents.value[index] = doc
    }
    else
    {
      documents.value.unshift(doc)
      totalCount.value++
    }
    currentDocument.value = doc
  }

  const updateDocumentStatus = (hash: string, status: DocumentStatus, has_embeddings: boolean) =>
  {
    const index = documents.value.findIndex(d => d.document_hash === hash)
    if (index !== -1)
    {
      documents.value[index].status = status
      documents.value[index].has_embeddings = has_embeddings
    }
  }

  const clearCurrentDocument = () =>
  {
    currentDocument.value = null
  }

  return {
    documents: computed(() => documents.value),
    currentDocument: computed(() => currentDocument.value),
    totalCount: computed(() => totalCount.value),
    loading: computed(() => loading.value),

    loadDocuments,
    deleteDocument,
    addOrUpdateDocument,
    updateDocumentStatus,
    clearCurrentDocument,
  }
}
