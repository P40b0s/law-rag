import { ModelStateServiceService } from './models_service';
import { DocumentsService } from './documents_service';

class HttpService
{
    public model_state_service: ModelStateServiceService;
    public documents_service: DocumentsService;

    constructor()
    {
        this.model_state_service = new ModelStateServiceService();
        this.documents_service = new DocumentsService();
    }
}
const http_sevice = new HttpService();
export { http_sevice };