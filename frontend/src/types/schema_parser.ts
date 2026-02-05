import { Err, Ok, Result } from "@globalart/oxide";
import { ZodType } from "zod/v3"

export function schema_parser<T, S extends ZodType<any, any>>(schema: S, data: unknown): Result<T, Error>
{
    const parsed = schema.safeParse(data);
    if(parsed.error)
    {
        console.log(parsed.error);
        return Err(parsed.error);
    }
    else
    {
        return Ok(parsed.data);
    }
}