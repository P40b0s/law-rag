//import mitt, {type Emitter} from 'mitt'
import { Task, TaskEvent, TaskDeleteEvent } from '@/types/task';
import { Emitter } from 'strict-event-emitter'
const emitter = new Emitter<Events>()
export default emitter
export {type Events, type Emitter};


type Events =
{
    test: [string]
    update_profile: []
    delete_packet:[string]
    open_pdf: [number],
    add_task: [TaskEvent],
    edit_task: [TaskEvent]
    delete_task: [TaskDeleteEvent]
};