import { z} from "zod"
import { date_time_schema } from "./date_schema"
import { DocumentSchema } from "./document";

const PacketSchema = z.object({
    id: z.string(),
    packet_dir: z.string(),
    created: date_time_schema,
    files_count: z.number(),
    user_id: z.number()
});

const FileSchema = z.object({
    packet_id: z.string(),
    name: z.string(),
    extension: z.string(),
    hash: z.string(),
});
const PacketsSchema = z.array(PacketSchema);
const FilesSchema = z.array(FileSchema);
const FileWithDocumentSchema = z.object({
    file: FileSchema,
    document: DocumentSchema.nullable()
})
const FilesWithDocumentSchema = z.array(FileWithDocumentSchema);
type PacketFile = z.infer<typeof FileSchema>;
type PacketFiles =  z.infer<typeof FilesSchema>;
type Packet = z.infer<typeof PacketSchema>;
type Packets = z.infer<typeof PacketsSchema>;
type FileWithDocument = z.infer<typeof FileWithDocumentSchema>;
type FilesWithDocument = z.infer<typeof FilesWithDocumentSchema>
export {
    type Packet,
    PacketSchema,
    type Packets,
    PacketsSchema,
    type PacketFile,
    FileSchema,
    type PacketFiles,
    FilesSchema,
    type FileWithDocument,
    type FilesWithDocument,
    FileWithDocumentSchema,
    FilesWithDocumentSchema
}