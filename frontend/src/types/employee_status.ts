import { z } from 'zod';


const EmployeeStatusSchema = z.object({
  id: z.uuidv7(),
  status: z.string(),
  description: z.string(),
  is_disease: z.boolean(),
  tracing: z.boolean(),
  on_work_place: z.boolean(),
  logo: z.string().nullish(),
  color: z.string()
});
type EmployeeStatus = z.infer<typeof EmployeeStatusSchema>;
export {type EmployeeStatus, EmployeeStatusSchema}