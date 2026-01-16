
import { http_sevice } from '@/services/http_service/http_service';
import { type PacketFiles } from '@/types/packet';
import { NTooltip } from 'naive-ui';
import { defineAsyncComponent, defineComponent, h, onMounted, PropType, ref } from 'vue';
import {type Document} from '@/types/document';
import StatusesList from './documents/StatusesList.vue';
//такая конструкция нормально работает в шаблоне с suspense
// suspense
//     template(#default)
//         async-file-document-map(:files="files_group")
//     template(#fallback)
//         div Loading...
const props = 
{
    files:
    {
        type: Object as PropType<PacketFiles>, 
        required: true
    }
};
const FileDocumentMap = defineComponent({
    props,
    async setup(props) 
    {
        const content = ref();
        const document = ref<Document>();
        const render_header = async () =>
        {
            if(props.files)
            {
                let pdf = props.files.find(f=>f.extension == 'pdf')
                if(pdf)
                {
                    let doc = await http_sevice.get_file_document_mapping(pdf.hash);
                    if(doc)
                    {
                        document.value = doc;
                        return h(StatusesList,
                            {
                                redaction: doc.complex_information.redaction,
                                checked: doc.checked
                            }
                        )
                        //return h('div', document.value.complex_information.doc_image_path);
                    }
                }
            }
            return h('div', "Документ не найден в базе данных");
        }
        content.value = await render_header();
        return {content}
    },
    render()
    {
        return this.content;
    }
});
export const AsyncFileDocumentMap = defineAsyncComponent({
    loader: async () => 
    {
      // Можно выполнить дополнительные асинхронные операции
      //await preloadDependencies()
      
      // Возвращаем наш компонент с асинхронным setup
      return FileDocumentMap
    },
    loadingComponent: {
      render: () => h('div', 'Загрузка...')
    },
    errorComponent: {
      render: () => h('div', 'Ошибка загрузки!')
    },
    delay: 200,
    timeout: 3000
  })

