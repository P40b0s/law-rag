import z from "zod/v3";

const ModelsStateSchema = z.object({
    retriver: z.boolean(), //загружена ли модель реранкера
    reranker: z.boolean(), //загружена ли модель ретривера
    generator: z.boolean(), //загружена ли модель генератора
    system_prompt: z.string().optional(), //определен ли системный промпт и если определен то какой
    model_size: z.number().optional() //размер модели в байтах (если загружена)
  
})

export type ModelsState = z.infer<typeof ModelsStateSchema>
export {ModelsStateSchema}
