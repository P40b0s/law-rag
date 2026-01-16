import { z } from "zod";
const roles = ['Administrator', 'User', 'Undefined'] as const;
type Role = typeof roles[number];
// Схема для Role (аналог enum Role)
const RoleSchema = z.enum(roles);

export {type Role, RoleSchema, roles}