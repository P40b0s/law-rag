import { DateTime, type BackendDateTime } from "@/services/date";
import z from "zod/v3";

// Схема для BackendDateTime (формат бэкенда)
export const backend_date_time_schema = z.object({
    year: z.number().int().min(1900).max(2100),
    month: z.number().int().min(1).max(12),
    day: z.number().int().min(1).max(31),
    hour: z.number().int().min(0).max(23),
    minute: z.number().int().min(0).max(59),
    second: z.number().int().min(0).max(59)
}) satisfies z.ZodType<BackendDateTime>;

// Схема для общей работы с датами (поддерживает разные форматы)
export const date_time_schema = z.custom<Date | string | number | BackendDateTime | null | undefined>()
    .refine((val) => 
    {
        if (val === null || val === undefined) 
        {
            return true;
        }
        try 
        {
            const date = DateTime.parse(val);
            return !isNaN(date.date.getTime());
        } 
        catch 
        {
            return false;
        }
    }, 
    {
        message: "Неправильный формат даты"
    })
    .transform((val) => 
    {
        if (val === null || val === undefined) 
        {
            return null;
        }
        return DateTime.parse(val);
    });