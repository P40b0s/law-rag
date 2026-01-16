/// <reference types="@rsbuild/core/types" />

declare module "*.vue" 
{
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}


interface ImportMetaEnv 
{
  // import.meta.env.PUBLIC_FOO
  readonly PUBLIC_API_ADDRESSE: string;
  readonly PUBLIC_API_PORT: number;
  readonly PUBLIC_API_VERSION: string;
  readonly PUBLIC_DEFAULT_AVATAR: string;
}

interface ImportMeta 
{
  readonly env: ImportMetaEnv;
}