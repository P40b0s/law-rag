import { ModelStateServiceService } from './models_service';
import { DocumentsService } from './documents_service';
import { CollectionsService } from './collections_service';

class HttpService
{
    public model_state_service: ModelStateServiceService;
    public documents_service: DocumentsService;
    public collections_service: CollectionsService;

    constructor()
    {
        this.model_state_service = new ModelStateServiceService();
        this.documents_service = new DocumentsService();
        this.collections_service = new CollectionsService();
    }
}
const http_service = new HttpService();
export { http_service };