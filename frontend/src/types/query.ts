import z from "zod/v3";

// Схема для результата поиска QdrantContext
const QdrantContextSchema = z.object({
  original_score: z.number(),
  reranker_score: z.number(),
  id: z.string(),
  text: z.string(),
  document_uri: z.string(),
  document_hash: z.string(),
  document_title: z.string(),
  document_number: z.string(),
  document_sign_date: z.string(),
  path: z.string(),
});

// Схема для массива результатов
const QdrantContextArraySchema = z.array(QdrantContextSchema);

// Схема для запроса поиска
const QueryRequestSchema = z.object({
  query: z.string().min(1, "Запрос не может быть пустым"),
  per_collection_limit: z.number().int().positive().default(10),
  final_limit: z.number().int().positive().default(5)
});

// Экспорт типов
export type QdrantContext = z.infer<typeof QdrantContextSchema>;
export type QueryRequest = z.infer<typeof QueryRequestSchema>;

// Экспорт схем
export {
  QdrantContextSchema,
  QdrantContextArraySchema,
  QueryRequestSchema
};
