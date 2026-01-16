// composables/useEmployeeStatusHistory.ts
import { ref, computed } from 'vue'
import type { Ref } from 'vue'
import { type EmployeeStatus } from '@types/employee_status'
import { http_sevice } from '@/services/http_service/http_service'
import { ClinicDictionary } from '@/types/clinic_dictionary'
import { Dictionary, DictionaryWithWeight } from '@/types/dictionary'
import { notify_service } from '@/services/notification_service'


const clinics: Ref<Map<string, ClinicDictionary>> = ref(new Map())
const departments: Ref<Map<string, DictionaryWithWeight>> = ref(new Map())
const statuses: Ref<Map<string, EmployeeStatus>> = ref(new Map())
const properties: Ref<Map<string, Dictionary>> = ref(new Map())
const loading = ref(false)
export const useDictionaries = () => 
{
  // Reactive state
 
  const error = ref<string | null>(null)

  // Computed properties
  const clinicOptions = computed(() => 
    Array.from(clinics.value.values()).map(cl => ({
      label: cl.name,
      value: cl.id
    }))
  )

  const statusOptions = computed(() =>
    Array.from(statuses.value.values()).map(status => ({
      label: status.status,
      value: status.id
    }))
  )

  const departmentOptions = computed(() =>
    Array.from(departments.value.values()).map(dep => ({
      label: dep.value,
      value: dep.id
    }))
  )

    const propertyOptions = computed(() =>
      Array.from(properties.value.values()).map(dep => ({
      label: dep.value,
      value: dep.id
    }))
  )

  // Methods
  const get_clinic = (clinicId: string): ClinicDictionary | undefined => clinics.value.get(clinicId);
  const get_status = (statusId: string): EmployeeStatus | undefined => statuses.value.get(statusId);
  const get_department = (departmentId: string): DictionaryWithWeight | undefined => departments.value.get(departmentId);
  const get_property = (id: string): Dictionary | undefined => properties.value.get(id);

  const load_dictionaries_data = async (): Promise<void> => 
  {
    try 
    {
      loading.value = true
      error.value = null
      const [clinicsData, statusesData, departmentsData, propertiesData] = await Promise.all([
        http_sevice.clinic_service.get_clinics(),
        http_sevice.employee_status_service.get(),
        http_sevice.department_service.get(),
        http_sevice.properties_service.get()
      ])

      const newStatusesMap = new Map<string, EmployeeStatus>()
      statusesData.forEach(status => newStatusesMap.set(status.id, status))
      statuses.value = newStatusesMap;

      const newClinicsMap = new Map<string, ClinicDictionary>()
      clinicsData.forEach(clinic => newClinicsMap.set(clinic.id, clinic))
      clinics.value = newClinicsMap;

      const newDepatmentsMap = new Map<string, DictionaryWithWeight>()
      departmentsData.forEach(dep => newDepatmentsMap.set(dep.id, dep))
      departments.value = newDepatmentsMap;

      const newPropertiesMap = new Map<string, Dictionary>()
      propertiesData.forEach(dep => newPropertiesMap.set(dep.id, dep))
      properties.value = newPropertiesMap;

    } 
    catch (err) 
    {
      error.value = err instanceof Error ? err.message : 'Неизвестная ошибка'
      throw err
    } 
    finally 
    {
      loading.value = false
    }
  }
  //отделы
  const edit_department = async (id: string, value: string, weight: number) =>
  {
    const edited = await http_sevice.department_service.edit(id, value, weight)
    if (edited) 
    {
      departments.value.set(id, edited);
      notify_service.notify_success('Отдел обновлен', '')
    }
  }
  const add_department = async (value: string, weight: number) =>
  {
    const added = await http_sevice.department_service.add(value, weight)
    if (added) 
    {
      departments.value.set(added.id, added);
      notify_service.notify_success('Отдел добавлен', '')
    }
  }
  const delete_department = async (id: string) =>
  {
    const added = await http_sevice.department_service.delete(id)
    
      departments.value.delete(id);
      notify_service.notify_success('Отдел удален', '')
  }
  //статусы
  const edit_status = async (id: string, status: string, description: string, is_disease: boolean, tracing: boolean, on_work_place: boolean, color: string, logo?: string) =>
  {
    const edited = await http_sevice.employee_status_service.edit(id, status, description, is_disease, tracing, on_work_place, color, logo)
    if (edited) 
    {
      statuses.value.set(id, edited);
      notify_service.notify_success('Статус обновлен', '')
    }
  }
  const add_status = async (status: string, description: string, is_disease: boolean, tracing: boolean, on_work_place: boolean, color: string, logo?: string) =>
  {
    const added = await http_sevice.employee_status_service.add(status, description, is_disease, tracing, on_work_place, color, logo)
    if (added) 
    {
      statuses.value.set(added.id, added);
      notify_service.notify_success('Статус добавлен', '')
    }
  }
  const delete_status = async (id: string) =>
  {
    const added = await http_sevice.employee_status_service.delete(id)
    
      statuses.value.delete(id);
      notify_service.notify_success('Статус удален', '')
  }
  //поликлинники
  const edit_clinic = async (id: string, name: string, addresse: string) =>
  {
    const edited = await http_sevice.clinic_service.edit(id, name, addresse)
    if (edited) 
    {
      clinics.value.set(id, edited);
      notify_service.notify_success('Поликлинника обновлена', '')
    }
  }
  const add_clinic = async (name: string, addresse: string) =>
  {
    const added = await http_sevice.clinic_service.add(name, addresse)
    if (added) 
    {
      clinics.value.set(added.id, added);
      notify_service.notify_success('Поликлинника добавлена', '')
    }
  }
  const delete_clinic = async (id: string) =>
  {
    const added = await http_sevice.clinic_service.delete(id)
    
      clinics.value.delete(id);
      notify_service.notify_success('Поликлинника удалена', '')
  }
  //свйоства сотрудника
  const edit_property = async (id: string, value: string) =>
  {
    const edited = await http_sevice.properties_service.edit(id, value)
    if (edited) 
    {
      properties.value.set(id, edited);
      notify_service.notify_success('Свойство обновлено', '')
    }
  }
  const add_property = async (value: string) =>
  {
    const added = await http_sevice.properties_service.add(value)
    if (added) 
    {
      properties.value.set(added.id, added);
      notify_service.notify_success('Свойство добавлено', '')
    }
  }
  const delete_property = async (id: string) =>
  {
    const added = await http_sevice.properties_service.delete(id)
    
      properties.value.delete(id);
      notify_service.notify_success('Свойство удалено', '')
  }

  const reload_data = async () => 
  {
    reset();
    await load_dictionaries_data();
  }

  // Reset state
  const reset = (): void => 
  {
    clinics.value = new Map()
    statuses.value = new Map()
    departments.value = new Map()
    loading.value = false
    error.value = null
  }

  return {
    // State
    clinics,
    departments,
    statuses,
    properties,
    loading,
    error,

    // Computed
    clinicOptions,
    statusOptions,
    departmentOptions,
    propertyOptions,

    // Methods
    get_clinic,
    get_department,
    get_status,
    get_property,
    edit_clinic,
    edit_department,
    edit_status,
    edit_property,
    add_clinic,
    add_department,
    add_status,
    add_property,
    delete_clinic,
    delete_department,
    delete_status,
    delete_property,
    load_dictionaries_data,

    reload_data,
    reset
  }
}

// Types for better DX
export type UseDictionariesReturn = ReturnType<typeof useDictionaries>