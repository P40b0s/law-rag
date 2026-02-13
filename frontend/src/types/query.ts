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
  chunk_index: z.number().int().nonnegative()
});

// Схема для массива результатов
const QdrantContextArraySchema = z.array(QdrantContextSchema);

// Схема для запроса поиска
const QueryRequestSchema = z.object({
  query: z.string().min(1, "Запрос не может быть пустым"),
  limit: z.number().int().positive().default(10),
  reranker_limit: z.number().int().positive().default(5)
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
