import { ref, computed, onMounted } from 'vue'
import { http_service } from '@/services/http_service/http_service'
import { Collection } from '@/types/collection'
import { Document } from '@/types/document'

// Глобальное состояние коллекций
const collections = ref<Collection[]>([])
const loading = ref(false)
const initialized = ref(false)

export default function useCollections() 
{
  /**
   * Загрузка списка коллекций
   */
  const loadCollections = async () => {
    loading.value = true
    try {
      const result = await http_service.collections_service.get_collections()
      if (result) {
        collections.value = result
      }
    } finally {
      loading.value = false
      initialized.value = true
    }
  }

  /**
   * Добавление новой коллекции
   */
  const addCollection = async (name: string, description: string): Promise<Collection | undefined> => {
    const newCollection = await http_service.collections_service.add_collection(name, description)
    if (newCollection) {
      collections.value.push(newCollection)
    }
    return newCollection
  }

  /**
   * Обновление коллекции
   */
  const updateCollection = async (id: string, description: string): Promise<boolean> => {
    const success = await http_service.collections_service.update_collection(id, description)
    if (success) 
    {
      const index = collections.value.findIndex(c => c.id === id)
      if (index !== -1) 
      {
        collections.value[index].description = description
      }
    }
    return success
  }

  /**
   * Удаление коллекции
   */
  const deleteCollection = async (id: string): Promise<boolean> => {
    const success = await http_service.collections_service.delete_collection(id)
    if (success) {
      const index = collections.value.findIndex(c => c.id === id)
      if (index !== -1) {
        collections.value.splice(index, 1)
      }
    }
    return success
  }

  /**
   * Получение коллекции по ID
   */
  const getCollectionById = (id: string): Collection | undefined => {
    return collections.value.find(c => c.id === id)
  }

  /**
   * Получение имени коллекции по ID
   */
  const getCollectionName = (id: string): string => {
    const collection = getCollectionById(id)
    return collection?.name ?? 'Неизвестная коллекция'
  }

  /**
   * Маппинг документа на коллекцию
   */
  const getDocumentCollection = (document: Document): Collection | undefined => {
    return getCollectionById(document.collection_id)
  }

  /**
   * Группировка документов по коллекциям
   */
  const groupDocumentsByCollection = (documents: Document[]): Map<string, Document[]> => {
    const grouped = new Map<string, Document[]>()

    for (const doc of documents) {
      if (!grouped.has(doc.collection_id)) {
        grouped.set(doc.collection_id, [])
      }
      grouped.get(doc.collection_id)!.push(doc)
    }

    return grouped
  }

  /**
   * Опции для select компонентов
   */
  const collectionOptions = computed(() => {
    return collections.value.map(collection => ({
      label: collection.name,
      value: collection.id,
      description: collection.description
    }))
  })

  /**
   * Инициализация при монтировании
   */
  onMounted(async () => {
    if (!initialized.value) {
      await loadCollections()
    }
  })

  return {
    // Состояние
    collections: computed(() => collections.value),
    loading: computed(() => loading.value),
    initialized: computed(() => initialized.value),
    collectionOptions,

    // Методы
    loadCollections,
    addCollection,
    updateCollection,
    deleteCollection,
    getCollectionById,
    getCollectionName,
    getDocumentCollection,
    groupDocumentsByCollection
  }
}
