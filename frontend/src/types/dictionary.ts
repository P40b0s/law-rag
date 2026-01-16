import { z } from 'zod';
const DictionarySchema = z.object({
  id: z.uuidv7(),
  value: z.string()
});
const DictionaryWithWeightSchema = z.object({
  weight: z.number(),
}).extend(DictionarySchema.shape);
type Dictionary = z.infer<typeof DictionarySchema>;
type DictionaryWithWeight = z.infer<typeof DictionaryWithWeightSchema>;


export {type Dictionary, type DictionaryWithWeight, DictionarySchema, DictionaryWithWeightSchema}