import { z } from 'zod';


const ClinicDictionarySchema = z.object({
  id: z.string(),
  name: z.string(),
  addresse: z.string(),
});
type ClinicDictionary = z.infer<typeof ClinicDictionarySchema>;
export {type ClinicDictionary, ClinicDictionarySchema}