import emitter from "@/services/emitter";
import {api_path} from '@/services/http_service/http_client'
import { ServiceStatusSchema } from "@/types/service_status";
import { Task, TaskDeleteSchema, TaskEventSchema, TaskSchema } from "@/types/task";
import { match } from "@globalart/oxide";
class EventSourceService
{
    evtSource: EventSource;
    constructor(uri: string)
    {
        console.log(uri)
        this.evtSource = new EventSource(uri);
        this.evtSource.onmessage = (event) => 
        {      
            console.log(`message: ${event.data}`);
        };
        this.evtSource.addEventListener("command", (c)=>
        {
            let cmd: Commands =  JSON.parse(c.data); 
            console.log(`Получена команда: ${cmd.event} + контент: `, cmd.content);
            match(cmd.event,
                [
                    ["status_message", () =>
                        {
                            const status = ServiceStatusSchema.parse(cmd.content);
                            emitter.emit('service_status', status);
                        }
                    ],
                () => "Неизвестная команда"
                ]
            );
        })
    }
} 

type SSECommands = "status_message";
type Commands = 
{
    event: SSECommands,
    content: unknown
}

//Получена команда: {"event":"update_documents_from_portal","content":[{"id":"asdasdqwd","eo_number":"0983490283923","complex_name":"sjdfoi sdjfoisj osdifjo jsdofij oisjdfosidjf osidjfos jdosdijf ","pages_count":5,"curr_page":0,"pdf_file_length":0,"publish_date_short":"2024-05-24T00:00:00","target_redactions":null}]} 2 event_source.ts:12:20

const sse_service = new EventSourceService(api_path + "sse")
export default sse_service;