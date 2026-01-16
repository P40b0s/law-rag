import { z } from 'zod';
import { RoleSchema } from "./user_role";
import { PermissionSchema } from "./permission";

export const avatar_parse_schema = z.custom<Uint8Array|undefined>()
.transform((val) => {
  if(val)
    new Uint8Array(val)
  else
    val
});
const UsernameSchema = z.object({
  username: z.string(),
});
const AvatarSchema = z.object({
  avatar: avatar_parse_schema,
});
const PasswordSchema = z.object({
  password: z.string().nullish(),
});
const TokenSchema = z.object({
  token: z.string().nullish(),
});
const UserInfoBaseSchema = z.object({
  id: z.uuidv7(), // u8 в Rust - число от 0 до 255
  first_name: z.string(),
  second_name: z.string(),
  surname: z.string(),
  role: RoleSchema,
  permissions: z.array(PermissionSchema)
});

/**
 * Схема для создания нового пользователя
 */
const CreateUserSchema = z.object({
  first_name: z.string(),
  second_name: z.string(),
  surname: z.string(),
  role: RoleSchema.nullish(),
  permissions: z.array(PermissionSchema).nullish()
})
.extend(UsernameSchema.shape)
.extend(PasswordSchema.shape)

const UserInfoUpdateSchema = z.object({
  id: z.uuidv7(), // u8 в Rust - число от 0 до 255
  first_name: z.string(),
  second_name: z.string(),
  surname: z.string(),
})
 .extend(PasswordSchema.shape);

const AdminUserInfoUpdateSchema = z.object({
  id: z.uuidv7(), // u8 в Rust - число от 0 до 255
  first_name: z.string(),
  second_name: z.string(),
  surname: z.string(),
  role: RoleSchema,
  permissions: z.array(PermissionSchema)
})
  .extend(PasswordSchema.shape);
  
// Основная схема для UserInfo
const UserLoginResponseSchema = UserInfoBaseSchema
  .extend(TokenSchema.shape)
  .extend(UsernameSchema.shape) //z.object({


type UserInfo = z.infer<typeof UserLoginResponseSchema>;
type UserInfoUpdate = z.infer<typeof UserInfoUpdateSchema>;
type AdminUserInfoUpdate = z.infer<typeof AdminUserInfoUpdateSchema>;
type CreateUserPayload = z.infer<typeof CreateUserSchema>;
export {type UserInfo, UserLoginResponseSchema, type UserInfoUpdate, UserInfoUpdateSchema, type CreateUserPayload , type AdminUserInfoUpdate, CreateUserSchema}

