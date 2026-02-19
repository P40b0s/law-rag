import z from "zod/v3";

// Схема для коллекций qdrant
const CollectionSchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  description: z.string()
})
const CollectionArraySchema = z.array(CollectionSchema);

// Экспорт типов
export type Collection = z.infer<typeof CollectionSchema>
export type CollectionArray = z.infer<typeof CollectionArraySchema>
// Экспорт схем
export {
  CollectionSchema,
  CollectionArraySchema
}