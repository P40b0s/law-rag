// types.ts
export interface DraggableItem {
  id: string | number
  [key: string]: any
}

export interface DragChangeEvent<T = DraggableItem> {
  added?: {
    newIndex: number,
    element: T
  }
  removed?: {
    oldIndex: number
    element: T
  }
  moved?: {
    newIndex: number
    oldIndex: number
    element: T
  }
}