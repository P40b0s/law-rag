import { z } from 'zod';


const EmployeesStatesCountsSchema = z.object({
  total_count: z.number(),
  all_diseases_count: z.number(),
  diseases_count: z.number(),
  tracing_diseases_count: z.number(),
  on_workplace_count: z.number(),
  total_worked_employees_count: z.number(),
 
});
type EmployeesStatesCounts = z.infer<typeof EmployeesStatesCountsSchema>;

const UuidSchema = z.string().uuid();
const FioSchema = z.object({
  id: UuidSchema,
  status_id: UuidSchema.nullish(),
  status: z.string(),
  surname: z.string(),
  first_name: z.string(),
  second_name: z.string(),
});

const StatisticGroupSchema = z.object({
  name: z.string(),
  status_counts: z.number().int().nonnegative(),
  employees_list: z.array(FioSchema),
});

const DepartmentStatisticSchema = z.object({
  department_name: z.string(),
  department_id: UuidSchema,
  groups: z.array(StatisticGroupSchema),
  employees_on_work: z.number().int().nonnegative(),
  employees_count: z.number().int().nonnegative(),
});

const DepartmentsStatisticSchema = z.object({
  departments: z.array(DepartmentStatisticSchema),
  total_on_work_count: z.number().int().nonnegative(),
  total_employees_count: z.number().int().nonnegative(),
  total_count_by_category: z.record(z.string(), z.number().int().nonnegative()),
});

// Типы TypeScript (опционально)
type Fio = z.infer<typeof FioSchema>;
type StatisticGroup = z.infer<typeof StatisticGroupSchema>;
type DepartmentStatistic = z.infer<typeof DepartmentStatisticSchema>;
type DepartmentsStatistic = z.infer<typeof DepartmentsStatisticSchema>;

export {type EmployeesStatesCounts, type Fio, type StatisticGroup, type DepartmentStatistic, type DepartmentsStatistic, DepartmentsStatisticSchema, EmployeesStatesCountsSchema}