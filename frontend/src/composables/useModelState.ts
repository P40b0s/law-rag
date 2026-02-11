import emitter from "@/services/emitter";
import { computed, readonly, ref, Ref, watch, type WatchStopHandle } from "vue";
import { ModelsState } from "@/types/models_state";
import { notify_service } from "@/services/notification_service";
import { http_service } from "@/services/http_service/http_service";

const state: Ref<ModelsState| null> = ref(null);
export default function useModelState()
{
    const set_state = (new_state: ModelsState) =>
    {
        state.value = new_state;
    }
    const get_state = () => readonly(state);
    const load_embedding_models = async () =>
    {
        const new_state = await http_service.model_state_service.load_embedding_models();
        if (new_state)
            set_state(new_state)
    }
    const unload_embedding_models = async () =>
    {
        const new_state = await http_service.model_state_service.unload_embedding_models();
        if (new_state)
            set_state(new_state)
    }
    const load_generator_model = async () =>
    {
        const new_state = await http_service.model_state_service.load_generator_model();
        if (new_state)
            set_state(new_state)
    }
    const unload_generator_model = async () =>
    {
        const new_state = await http_service.model_state_service.unload_generator_model();
        if (new_state)
            set_state(new_state)
    }
    
    return {
        set_state,
        get_state,
        load_embedding_models,
        unload_embedding_models,
        load_generator_model,
        unload_generator_model
    }
}