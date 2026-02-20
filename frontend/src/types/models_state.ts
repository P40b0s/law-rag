import z from "zod/v3";

const RetriverStateSchema = z.object({
    retriver_model_name: z.string(),
    reranker_model_name: z.string(),
})

const GeneratorStateSchema = z.object({
    model_name: z.string(),
    model_size: z.number(),
    system_prompt: z.string(),
    uri: z.string().nullable().optional(),
    temperature: z.number()
})

const ModelsStateSchema = z.object({
    retriver: RetriverStateSchema.optional(),
    generator: GeneratorStateSchema.optional(),
})

export type RetriverState = z.infer<typeof RetriverStateSchema>
export type GeneratorState = z.infer<typeof GeneratorStateSchema>
export type ModelsState = z.infer<typeof ModelsStateSchema>
export { ModelsStateSchema, RetriverStateSchema, GeneratorStateSchema }
