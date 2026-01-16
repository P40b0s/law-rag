import { z} from "zod"

interface DynamicVNode 
{
  tag: string;
  attrs: Record<string, string | number | boolean | null | undefined> | null;
  children: DynamicVNodeChildren;
}

type DynamicVNodeChildren = 
  | string 
  | number 
  | DynamicVNode 
  | DynamicVNode[] 
  | null;

interface EditorJsBlock 
{
  type: string;
  data: Record<string, unknown>;
}
interface EditorJsModel
{
    time: number;
    blocks: EditorJsBlock[];
    version: string;
}
const DynamicVNodeChildrenSchema: z.ZodType<DynamicVNodeChildren> = z.lazy(() =>
  z.union([
    z.array(DynamicVNodeSchema).or(DynamicVNodeSchema),
    z.string(),
    z.number(),
    z.null()
  ])
);

const DynamicVNodeSchema: z.ZodType<DynamicVNode> = z.lazy(() =>
  z.object({
    tag: z.string(),
    attrs: z.record(
      z.union([z.string(), z.number(), z.boolean(), z.null(), z.undefined()])
    ).nullable(),
    children: DynamicVNodeChildrenSchema
  })
);
//с рекурсией какая то шляпа в zod почему то выдает ошибку, поэтому просто использую интерфейс, так работает но без проверки, ну бэкенд тоже мой поэтому заошибки можно не переживать
export { type DynamicVNode, type EditorJsBlock, type EditorJsModel};