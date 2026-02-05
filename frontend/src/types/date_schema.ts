import { DateTime } from "@/services/date";
import z from "zod/v3";

// export const date_time_schema = z.custom<Date|string|number>().refine((val) => 
//     {
//         console.log(val)
//         const date = DateTime.parse(val);
//         return !isNaN(date.date.getTime());
//     }, 
//     {
//         message: "Неправильный формат даты"
//     })
// .transform((val) => DateTime.parse(val));
export const date_time_schema = z.custom<Date | string | number | null | undefined>()
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