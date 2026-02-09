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
    "Complete",
    "Error"
  ]),
  message: z.string()
})

export type ServiceStatus = z.infer<typeof ServiceStatusSchema>
export {ServiceStatusSchema}
