import { z } from 'zod';

// Схема для ChunkMeta
const ChunkMetaSchema = z.object({
  chunk_index: z.number().int().nonnegative(),
  token_count: z.number().int().nonnegative()
});

// Схема для Chunk
const ChunkSchema = z.object({
  publication_url: z.string(),
  document_url: z.string(),
  title: z.string(),
  number: z.string(),
  sign_date: z.string().refine((val) => !isNaN(Date.parse(val)), {
    message: "Invalid date format"
  }),
  hash: z.string(),
  path: z.string(),
  content: z.string(),
  links_hashes: z.array(z.string()).nullable().optional(),
  embeddings: z.array(z.number()).nullable().optional(),
  meta: ChunkMetaSchema.nullable().optional()
});

// Enum для LoadStatus
const LoadStatusSchema = z.enum(["NotFound", "Timeout", "Complete", "Pending"]);

// Типы TypeScript для LoadStatus
type LoadStatus = z.infer<typeof LoadStatusSchema>;

// Схема для FrontendDocument
const DocumentSchema = z.object({
  date: z.string().refine((val) => !isNaN(Date.parse(val)), {
    message: "Invalid date format"
  }),
  number: z.string(),
  first_chunk: ChunkSchema.nullable().optional(),
  status: LoadStatusSchema
});
// Для использования с массивом
const DocumentsArraySchema = z.array(DocumentSchema);
const ChunksArraySchema = z.array(ChunkSchema);

// Экспорт типов TypeScript
type Chunk = z.infer<typeof ChunkSchema>;
type ChunkMeta = z.infer<typeof ChunkMetaSchema>;
type Document = z.infer<typeof DocumentSchema>;

// Экспорт схем
export {
  ChunkMetaSchema,
  ChunkSchema,
  LoadStatusSchema,
  DocumentSchema,
  DocumentsArraySchema,
  ChunksArraySchema,
};

// Экспорт типов
export type {
  Chunk,
  ChunkMeta,
  Document,
  LoadStatus
};