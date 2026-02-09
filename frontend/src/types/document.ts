import z from "zod/v3";
import { date_time_schema } from "./date_schema"

// Схема для ChunkMeta
const ChunkMetaSchema = z.object({
  chunk_index: z.number().int().nonnegative(),
  token_count: z.number().int().nonnegative(),
})

// Схема для Chunk
const ChunkSchema = z.object({
  publication_url: z.string(),
  document_url: z.string(),
  title: z.string(),
  number: z.string(),
  sign_date: date_time_schema,
  hash: z.string(),
  path: z.string(),
  content: z.string(),
  links_hashes: z.array(z.string()).optional(),
  embeddings: z.array(z.number()).optional(),
  meta: ChunkMetaSchema.optional(),
})

// Enum для статуса документа
const DocumentStatusSchema = z.enum(["NotLoaded", "Loaded", "Embedded", "Unknown"])

// Схема для Document
const DocumentSchema = z.object({
  document_uri: z.string(),
  document_hash: z.string(),
  document_title: z.string(),
  document_number: z.string(),
  document_sign_date: date_time_schema,
  has_embeddings: z.boolean(),
  chunks_count: z.number().int().nonnegative(),
  status: DocumentStatusSchema,
})

const DocumentArraySchema = z.array(DocumentSchema);

// Экспорт типов
export type ChunkMeta = z.infer<typeof ChunkMetaSchema>
export type Chunk = z.infer<typeof ChunkSchema>
export type DocumentStatus = z.infer<typeof DocumentStatusSchema>
export type Document = z.infer<typeof DocumentSchema>


// Экспорт схем
export {
  ChunkMetaSchema,
  ChunkSchema,
  DocumentStatusSchema,
  DocumentSchema,
  DocumentArraySchema
}