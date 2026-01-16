import { match } from "ts-pattern";
import emitter from "@/services/emitter";
import {api_path} from '@/services/http_service/http_client'
import { Task, TaskDeleteSchema, TaskEventSchema, TaskSchema } from "@/types/task";
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
            match(cmd.event)
                .with("update_profile", () =>
                {
                    emitter.emit('update_profile')
                })
                .with('delete_packet', () =>
                {
                    emitter.emit('delete_packet', cmd.content as string);
                })
                .with('add_task', () =>
                {
                    emitter.emit('add_task', TaskEventSchema.parse(cmd.content));
                })
                .with('edit_task', () =>
                {
                    emitter.emit('edit_task', TaskEventSchema.parse(cmd.content));
                })
                .with('delete_task', () =>
                {
                    emitter.emit('delete_task', TaskDeleteSchema.parse(cmd.content));
                })
                
        })
    }
} 

type SSECommands = "update_profile" | "delete_packet" | "add_task" | "edit_task" | "delete_task"
type Commands = 
{
    event: SSECommands,
    content: unknown
}

//Получена команда: {"event":"update_documents_from_portal","content":[{"id":"asdasdqwd","eo_number":"0983490283923","complex_name":"sjdfoi sdjfoisj osdifjo jsdofij oisjdfosidjf osidjfos jdosdijf ","pages_count":5,"curr_page":0,"pdf_file_length":0,"publish_date_short":"2024-05-24T00:00:00","target_redactions":null}]} 2 event_source.ts:12:20

const sse_service = new EventSourceService(api_path + "sse")
export default sse_service;