import emitter from "@/services/emitter";
import { computed, readonly, ref, Ref, watch, type WatchStopHandle } from "vue";
import { ModelsState } from "@/types/models_state";
import { notify_service } from "@/services/notification_service";

const state: Ref<ModelsState| null> = ref(null);
export default function useModelState()
{
    const set_state = (new_state: ModelsState) =>
    {
        state.value = new_state;
        console.log("Установлена новая модель", state.value.generator, state.value.reranker, state.value.retriver, state.value.model_size, state.value.system_prompt);
    }
    const get_state = () => readonly(state);
    
    return {set_state, get_state}
}