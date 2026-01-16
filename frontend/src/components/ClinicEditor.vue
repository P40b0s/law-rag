<template lang="pug">
n-card.clinic-manager(title="Управление поликлинниками")
  template(#header-extra)
    n-button(type="primary" @click="openAddModal")
      template(#icon)
        n-icon: add-icon
      | Добавить поликлиннику

  n-space(vertical :size="20")
    //- Поиск и фильтрация
    n-input(
      v-model:value="searchQuery"
      placeholder="Поиск по названию..."
      clearable
    )
      template(#prefix)
        n-icon: search-icon

    //- Список клиник
    n-list(bordered)
      n-list-item(v-for="clinic in filteredClinics" :key="clinic.id")
        template(#suffix)
          n-space
            n-tooltip Удалить
              template(#trigger)
                n-button(round text @click="confirmDelete(clinic)")
                  template(#icon)
                    n-icon(:size="25" color="#ec3c36"): TrashBin
        n-thing(:title="clinic.name" :description="clinic.addresse")
        template(#prefix)
          n-tooltip Редактировать
            template(#trigger)
              n-button(round text @click="openEditModal(clinic)")
                template(#icon)
                  n-icon(:size="25" color="#82e873"): EditIcon

    //- Пустое состояние
    n-empty(
      v-if="filteredClinics.length === 0"
      description="Поликлинники не найдены"
    )
      template(#extra)
        n-button(size="small" @click="openAddModal") Добавить  поликлиннику

//- Модальное окно добавления/редактирования
n-modal(
  v-model:show="showModal"
  :title="editingClinic ? 'Редактировать клинику' : 'Добавить поликлиннику'"
  preset="dialog"
  positive-text="Сохранить"
  negative-text="Отмена"
  @positive-click="handleSave"
  @negative-click="handleCancel"
)
  n-form(
    ref="formRef"
    :model="formModel"
    :rules="formRules"
    label-placement="top"
  )
    n-form-item(label="Название поликлинники" path="name")
      n-input(
        v-model:value="formModel.name"
        placeholder="Введите название поликлинники"
      )
    n-form-item(label="Адрес" path="addresse")
      n-input(
        v-model:value="formModel.addresse"
        placeholder="Введите адрес поликлинники"
        type="textarea"
        :autosize="{ minRows: 2, maxRows: 4 }"
      )

//- Диалог подтверждения удаления
n-modal(
  v-model:show="showDeleteConfirm"
  preset="dialog"
  type="error"
  title="Подтверждение удаления"
  content="Вы уверены, что хотите удалить эту поликлинники?"
  positive-text="Удалить"
  negative-text="Отмена"
  @positive-click="handleDelete"
  @negative-click="showDeleteConfirm = false"
)
</template>
<script lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  NCard,
  NButton,
  NIcon,
  NSpace,
  NInput,
  NList,
  NListItem,
  NThing,
  NAvatar,
  NEmpty,
  NModal,
  NForm,
  NTooltip,
  NFormItem,
  useMessage,
  type FormRules,
  type FormInst
} from 'naive-ui'
import { AddOutline as AddIcon, SearchOutline as SearchIcon, TrashBin} from '@vicons/ionicons5'
import { Edit as EditIcon } from '@vicons/carbon'
import { type ClinicDictionary } from '@/types/clinic_dictionary';
import { notify_service } from '@/services/notification_service';
import { http_sevice } from '@/services/http_service/http_service';
import { useDictionaries } from '../composables/useDictionaries';


</script>
<script lang="ts" setup>
const formRef = ref<FormInst | null>(null)
const {clinics, delete_clinic, add_clinic, edit_clinic} = useDictionaries();
console.log(clinics.value.values());
// Состояние
const clinics_ref = computed(() => Array.from(clinics.value.values()))
const searchQuery = ref('')
const showModal = ref(false)
const showDeleteConfirm = ref(false)
const editingClinic = ref<ClinicDictionary | null>(null)
const clinicToDelete = ref<ClinicDictionary | null>(null)

// Модель формы
const formModel = ref({
  name: '',
  addresse: ''
})

// Правила валидации
const formRules: FormRules = {
  name: [
    {
      required: true,
      message: 'Название поликлинники обязательно',
      trigger: ['blur', 'input']
    },
    {
      min: 2,
      message: 'Название должно содержать минимум 2 символа',
      trigger: ['blur', 'input']
    }
  ],
  addresse: [
    {
      required: true,
      message: 'Адрес обязателен',
      trigger: ['blur', 'input']
    }
  ]
}

// Отфильтрованные клиники
const filteredClinics = computed(() => 
{
  if (!searchQuery.value) 
  {
    return clinics_ref.value
  }

  const query = searchQuery.value.toLowerCase()
  return clinics_ref.value.filter(clinic =>
    clinic.name.toLowerCase().includes(query) ||
    clinic.addresse.toLowerCase().includes(query)
  )
})

// Загрузка данных (заглушка)
onMounted(async () => 
{
  //clinics_ref.value = clinics.value.values();
})


// Открытие модального окна для добавления
const openAddModal = () => 
{
  editingClinic.value = null
  formModel.value = 
  {
    name: '',
    addresse: ''
  }
  showModal.value = true
}

// Открытие модального окна для редактирования
const openEditModal = (clinic: ClinicDictionary) => 
{
  editingClinic.value = clinic
  formModel.value = 
  {
    name: clinic.name,
    addresse: clinic.addresse
  }
  showModal.value = true
}

// Подготовка к удалению
const confirmDelete = (clinic: ClinicDictionary) => 
{
  clinicToDelete.value = clinic
  showDeleteConfirm.value = true
}

// Удаление клиники
const handleDelete = async () => 
{
  if (clinicToDelete.value) 
  {
    //clinics.value = clinics.value.filter(c => c.id !== clinicToDelete.value!.id)
    //let del = await http_sevice.clinic_service.delete(clinicToDelete.value.id);
    //notify_service.notify_success('Поликлинники удалена', "")
    await delete_clinic(clinicToDelete.value.id);
    clinicToDelete.value = null
  }
  showDeleteConfirm.value = false
}

// Сохранение клиники
const handleSave = () => 
{
  formRef.value?.validate(async (errors) => {
    if (errors) 
    {
      notify_service.notify_error('Пожалуйста, исправьте ошибки в форме', "")
      return false
    }

    try 
    {
      if (editingClinic.value) 
      {
        await edit_clinic(editingClinic.value.id, formModel.value.name, formModel.value.addresse);
        //const edited = await http_sevice.clinic_service.edit(editingClinic.value.id, formModel.value.name, formModel.value.addresse);
        //if(edited)
        //{
            // Редактирование существующей клиники
            //const index = clinics.value.findIndex(c => c.id === editingClinic.value!.id)
            //if (index !== -1) 
            //{
            //clinics.value[index] = 
            //{
            //    ...edited,
                //name: formModel.value.name,
                //addresse: formModel.value.addresse
            //}
            
            //}
            //notify_service.notify_success('Поликлинники обновлена', "")
        //}
      } 
      else 
      {
        // Добавление новой клиники
        const newClinic: ClinicDictionary = 
        {
          id: "",
          name: formModel.value.name,
          addresse: formModel.value.addresse
        }
        //const added = await http_sevice.clinic_service.add(newClinic.name, newClinic.addresse);
        //if(added)
        //{
        //    clinics.value.push(added)
        //    notify_service.notify_success('Поликлинника добавлена', "")
        //}
        await add_clinic(formModel.value.name, formModel.value.addresse);
      }

      showModal.value = false
      resetForm()
      return true
    } 
    catch (error) 
    {
      notify_service.notify_error('Ошибка при сохранении поликлинники', "")
      return false
    }
  })
}

// Отмена редактирования
const handleCancel = () => {
  showModal.value = false
  resetForm()
}

// Сброс формы
const resetForm = () => {
  formModel.value = {
    name: '',
    addresse: ''
  }
  editingClinic.value = null
}

</script>
<style lang="scss" scoped>
.clinic-manager {
  max-width: 800px;
  min-width: 600px;
  margin: 0 auto;
}

.n-list 
{
  max-height: calc(100vh - 220px);
  overflow-y: auto;
}
.n-list-item {
  padding: 12px;
}
.add-button
{
  margin-left: 20px;
}
</style>