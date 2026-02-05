import z from "zod"
import { date_time_schema } from "./date_schema"
import Tasks from '@/views/Tasks.vue';
import { DateTime } from "@/services/date";

const TaskFileSchema = z.object(
{
  id: z.uuid(),
  task_id: z.uuid(),
  filename: z.string(),
  storage_path: z.string(),
  size: z.number().min(0),
  mime_type: z.string(),
  file: z.file().optional(),
  hash: z.string(),
})
const TaskFilesSchema = z.array(TaskFileSchema);

const TaskStageSchema = z.object(
  {
    name: z.string(),
    completed: z.boolean(),
    timestramp: z.number(),
  }
)

const TaskSchema = z.object(
{
  id: z.uuid().optional(),
  title: z.string(),
  description: z.string(),
  department_id: z.uuid().optional(),
  users: z.array(z.uuid()),
  status: z.enum(['todo', 'in_progress', 'done']),
  priority: z.enum(['low', 'medium', 'high']),
  target_date: date_time_schema,
  changed_by: z.tuple([z.uuid(), date_time_schema]).optional(),
  added_by: z.tuple([z.uuid(), date_time_schema]),
  note: z.string(),
  files: z.array(TaskFileSchema),
  tags: z.array(z.string()),
  task_stages: z.array(TaskStageSchema)
})

const TasksSchema = z.array(TaskSchema);

const TaskEventSchema = z.object(
{
  user_id: z.uuid(),
  task: TaskSchema,
})
const TaskDeleteSchema = z.object(
{
  user_id: z.uuid(),
  task_id: z.uuid(),
})

const TagsResponseSchema = z.object(
  {
    tags: z.array(z.string())
  }
);

type TaskFilter = {
  department?: string,
  dates_range?: [number, number]
  tag?: string,
  status?: string[]
}
export type TaskFile = z.infer<typeof TaskFileSchema>
export type TaskEvent = z.infer<typeof TaskEventSchema>
export type TaskDeleteEvent = z.infer<typeof TaskDeleteSchema>
export type Task = z.infer<typeof TaskSchema>
export type TaskStage = z.infer<typeof TaskStageSchema>
export type TagsResponse = z.infer<typeof TagsResponseSchema>
export { TaskSchema, TaskFileSchema, TasksSchema, TaskEventSchema, TaskFilesSchema, TagsResponseSchema, TaskDeleteSchema, type TaskFilter}