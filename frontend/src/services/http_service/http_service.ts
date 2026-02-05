import { ModelStateServiceService } from './models_service';

class HttpService
{
    public model_state_service: ModelStateServiceService;
    constructor()
    {
        this.model_state_service = new ModelStateServiceService();
    }
}
const http_sevice = new HttpService();
export { http_sevice };