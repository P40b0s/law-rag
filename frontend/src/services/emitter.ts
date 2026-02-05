//import mitt, {type Emitter} from 'mitt'
import { ServiceStatus } from '@/types/service_status';
import { Task, TaskEvent, TaskDeleteEvent } from '@/types/task';
import { Emitter } from 'strict-event-emitter'
const emitter = new Emitter<Events>()
export default emitter
export {type Events, type Emitter};


type Events =
{
    service_status: [ServiceStatus]
};