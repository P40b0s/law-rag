import z from "zod/v3";

const ServiceStatusSchema = z.object({
  hash: z.string(),
  current_chunk: z.number().int().optional(),
  overall_chunks: z.number().int().positive().optional(),
  status: z.enum([
    "Embedding",
    "Reranking",
    "Generation",
    "Chunking",
    "Message",
    "Error"
  ]),
  message: z.string(),
  system_prompt: z.string().optional(),
  model_size: z.number().optional(),
})

export type ServiceStatus = z.infer<typeof ServiceStatusSchema>
export {ServiceStatusSchema}
