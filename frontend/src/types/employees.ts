import { z } from 'zod';
import { date_time_schema } from './date_schema';

const EmployeeSchema = z.object({
  id: z.uuidv7(),
  first_name: z.string(),
  second_name: z.string(),
  surname: z.string(),
  birthday: date_time_schema,
  clinic_id: z.uuidv7(),
  department_id: z.uuidv7(),
  state_id: z.uuidv7().nullish(),
  status_id: z.uuidv7().nullish(),
  state_start_date: date_time_schema.nullable(),
  state_end_date: date_time_schema.nullable()
});

const ShortEmployeesCalendarSchema = z.object({
  id: z.uuidv7(),
  first_name: z.string(),
  second_name: z.string(),
  surname: z.string(),
  department: z.uuidv7(),
})



const IdSchema = z.object({
  id: z.uuidv7(),
});
const AddEmployeeSchema = z.object({
  first_name: z.string(),
  second_name: z.string(),
  surname: z.string(),
  birthday: date_time_schema,
  clinic_id: z.uuidv7(),
  department_id: z.uuidv7()
});

const AssotiatedEmployeeSchema = z.object({
  first_name: z.string(),
  second_name: z.string(),
  surname: z.string(),
  department: z.uuidv7()
})
.extend(IdSchema.shape).nullable();
const EmployeesListSchema = z.array(AssotiatedEmployeeSchema);

const UpdateEmployeeSchema = z.object({
})
.extend(AddEmployeeSchema.shape)
.extend(IdSchema.shape);

const EmployeesSchema = z.array(EmployeeSchema);


const EmployeeStateSchema = z.object({
  employee_id: z.string(),
  status_id: z.string(),
  start_date: date_time_schema,
  end_date: date_time_schema,
  note: z.string().nullish(),
})
.extend(IdSchema.shape);

const EmployeeNewStateSchema = z.object({
  employee_id: z.string(),
  status_id: z.string(),
  start_date: date_time_schema,
  end_date: date_time_schema,
  note: z.string().nullish(),
});

const CalendarSchema = z.object({
  employee: ShortEmployeesCalendarSchema,
  statuses: z.array(EmployeeStateSchema)
});
const CalendarArrayShema = z.array(CalendarSchema);


const EmployeeStatesSchema = z.array(EmployeeStateSchema);
type EmployeeState = z.infer<typeof EmployeeStateSchema>;
type EmployeeNewState = z.infer<typeof EmployeeNewStateSchema>;
type EmployeeStates = z.infer<typeof EmployeeStatesSchema>;
type Employee = z.infer<typeof EmployeeSchema>;
type Employees = Employee[];
type AddNewEmployee = z.infer<typeof AddEmployeeSchema>;
type UpdateEmployee = z.infer<typeof UpdateEmployeeSchema>;
type CalendarEmployee = z.infer<typeof CalendarSchema>;
type AssotiatedEmployee = z.infer<typeof AssotiatedEmployeeSchema>;

export {
  type Employee,
  type Employees,
  type AddNewEmployee,
  type UpdateEmployee,
  type EmployeeState,
  type EmployeeNewState,
  type EmployeeStates,
  type CalendarEmployee,
  type AssotiatedEmployee,
  EmployeeStateSchema,
  EmployeeStatesSchema,
  UpdateEmployeeSchema,
  AddEmployeeSchema,
  EmployeeSchema,
  EmployeesSchema,
  EmployeeNewStateSchema,
  CalendarArrayShema,
  AssotiatedEmployeeSchema,
  EmployeesListSchema
}