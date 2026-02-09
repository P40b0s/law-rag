import { notify_service } from '../notification_service';
import { Err, match, Ok, Result } from '@globalart/oxide';
import { schema_parser } from '@/types/schema_parser';
import { z, ZodType, ZodTypeDef } from 'zod/v3';

// Используем относительный путь для self-hosted приложений
// Автоматически работает на любом домене/порте
export const api_path = "/" + import.meta.env.PUBLIC_API_VERSION + "/";
class HTTPError extends Error {}
HTTPError.prototype.name = "HTPError";

// const gen_req_error = async (resp: Response): Promise<Err<HTTPError>> => 
// {
//     const err = await match(resp.status)
//     .with(406, async () => 
//     {
//         const text = await resp.text();
//         return new HTTPError(`${text}`)
//     })
//     .otherwise(async () => new HTTPError(`Ошибка получения данных от АПИ: ${resp.status}:${resp.text}`))
//     //200-299 status code
//     console.error(err);
//     return createErr(err);
// }
type ResponseError = {err: string}
const gen_req_error = async (resp: Response): Promise<Err<HTTPError>> => 
{
    const status = resp.status;
    const body = await resp.text();
    const body_message: ResponseError = JSON.parse(body);
    const err = new HTTPError(`Ошибка ${resp.status}: ${body_message.err}`)
    //200-299 status code
    console.error(err);
    notify_service.error(`Ошибка ${status}`, body_message.err)
    return Err(err);
}
const fetch_error = (error: unknown): Err<HTTPError> => 
{
    const err = new HTTPError(`Ошибка запроса: ${error}`);
    console.error(err);
    return Err(err);
}

const get = async <Q extends string, R, S extends ZodType<any, any>>(path_and_query: Q, zod_type?: S): Promise<Result<R, HTTPError>> => 
{
    try
    {
        const response = await fetch(api_path + path_and_query, 
        {
            method: 'GET',
            
        });
        if (!response.ok) 
        {
            return await gen_req_error(response);
        }
        else
        {
            return get_body(response, zod_type);
        }
    }
    catch (error)
    {
        return fetch_error(error);
    }
}

const get_body = <B, S extends ZodType<unknown, any>>(response: Response, schema?: S): Promise<Result<B, HTTPError>> =>
{
    const content_type = response.headers.get('Content-Type');
    const def = async () => { return Ok({} as B);}
    if(content_type)
    {
        const object = match(content_type,
            [
                ['application/json', async () => 
                {
                    if(schema)
                    {
                        
                        const json = await response.json();
                        console.log("Получен ответ json", json)
                        const obj: Result<B, Error> = schema_parser(schema, json);
                        if(obj.isErr())
                        {
                            console.error("Ошибка преобразования объекта из схемы json", obj.unwrapErr());
                            return Err(new HTTPError(JSON.stringify(obj.unwrapErr(), null, 2))); 
                        }
                        else
                        {
                            return Ok(obj.unwrap());
                        }
                    }
                    else
                    {
                        const json: B = await response.json();
                        return Ok(json);
                    }
                }],
                ['text/plain; charset=utf-8', async () => Ok(await response.text() as B)],
                ['multipart/form-data', async () => Ok(await response.blob() as B)],
                async () => Err(new HTTPError("Необработанный тип сообщения " + content_type + " -> " + await response.text()))

            ]
        )
        // .with('application/json', async () =>
        // {
        //     if(schema)
        //     {
        //         const json = await response.json();
        //         //try
        //         //{
        //         const validation = schema.safeParse(json);
        //         if(!validation.success)
        //         {
        //             console.error(JSON.stringify(validation.error, null, 2)); 
        //             return createErr(new HTTPError(JSON.stringify(validation.error, null, 2))); 
        //         }
        //         else
        //         {
        //             return createOk(validation.data);
        //         }
        //             //const parsedResponse: B = schema.parse(json);
                    
        //         //}
        //         // catch(error)
        //         // {
        //         //     if (error instanceof z.ZodError) 
        //         //     {
        //         //         // Полный вывод ошибки в формате JSON
        //         //         console.error(JSON.stringify(error, null, 2)); 
        //         //         return createErr(new HTTPError(JSON.stringify(error, null, 2))); 
        //         //     }
        //         //     return createErr(new HTTPError(error as string));
        //         // }
        //     }
        //     else
        //     {
        //         const json: B = await response.json();
        //         return createOk(json);
        //     }
        // })
        // .with('text/plain; charset=utf-8', async () =>
        // {
        //     return createOk(await response.text() as B);
        // })
        // .with('multipart/form-data', async () =>
        // {
        //     return createOk(await response.blob() as B);
        // })
        // .otherwise(async () => createErr(new HTTPError("Необработанный тип сообщения " + content_type + " -> " + await response.text())));
        return object;
    }
    else
    {
        return def();
    }
}
// (type === 'object') {
//     if (Object.keys(v).length < 1) {
//         return true;
/**
 * const tt: Result<string, HTTPError> = await post("", 'PATCH')
 */
const post = async <Q extends string, P, R extends unknown, S extends ZodType, H extends Record<string, string>>(path_and_query: Q, method: "PATCH"|"POST", payload?: P, schema?: S, headers?: H): Promise<Result<R, HTTPError>> => 
{
    try
    {
        let local_headers = new Headers(headers);
        // if(headers)
        // {
        //     Object.entries(headers).forEach(h=>
        //     {
        //         local_headers.append(h[0], h[1]);
        //     })
        // }
        if(payload)
        {
            let is_form_data = isFormData(payload);
            if(!is_form_data)
                local_headers.append('content-type', 'application/json')
            const body = is_form_data ? (payload as unknown) as FormData : JSON.stringify(payload);
            const response = await fetch(api_path + path_and_query, 
            {
                method: method,
                body: body,
                headers: local_headers
            });
            if (!response.ok) 
            {
                return await gen_req_error(response);
            }
            else
            {
                return get_body(response, schema);
            }
        }
        else
        {
            const response = await fetch(api_path + path_and_query, 
            {
                method: method,
                headers: local_headers
            });
            if (!response.ok) 
            {
                return await gen_req_error(response);
            }
            else
            {
                return Ok({} as R);
            }
        }
    }
    catch (error)
    {
        return fetch_error(error);
    }
}

function isFormData(obj: unknown): obj is FormData 
{
    return obj instanceof FormData;
}
export {post, get, HTTPError}