import { z } from "zod";
const permissions = ["All", "Duty", "Boss", "View"] as const;
type Permission = typeof permissions[number];
// Схема для Privilegy (аналог enum Privilegy)
const PermissionSchema = z.enum(permissions);


export {type Permission, PermissionSchema, permissions}