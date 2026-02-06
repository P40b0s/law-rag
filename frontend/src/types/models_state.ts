import z from "zod/v3";

const ModelsStateSchema = z.object({
    retriver: z.boolean(), //загружена ли модель реранкера
    generator: z.boolean(), //загружена ли модель генератора
    system_prompt: z.string(), //системный промпт
    model_size: z.number() //размер модели в байтах 
  
})

export type ModelsState = z.infer<typeof ModelsStateSchema>
export {ModelsStateSchema}
