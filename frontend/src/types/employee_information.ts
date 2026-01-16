import { z } from 'zod';


const EmployeeInformationSchema = z.object({
  id: z.uuidv7(),
  employee_id: z.uuidv7(),
  property:  z.uuidv7(),
  value: z.string(),
});
const EmployeesInformationSchema = z.array(EmployeeInformationSchema);
type EmployeeInformation = z.infer<typeof EmployeeInformationSchema>;
export {type EmployeeInformation, EmployeeInformationSchema, EmployeesInformationSchema}