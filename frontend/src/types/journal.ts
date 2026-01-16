export type JournalItem = 
{
    time: string,
    note: string
}
export type Journal = 
{
    date: string,
    items: JournalItem[]
}