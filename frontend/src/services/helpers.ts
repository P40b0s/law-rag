import {  h, type RendererElement, type RendererNode, type VNode, type Ref } from 'vue';
import { DateFormat, DateTime } from './date';

const sleepNow = (delay: number) => new Promise((resolve) => setTimeout(resolve, delay))
const timer  =  (delay: number) =>  setTimeout(()  => { }, delay);

let tm : NodeJS.Timeout = setTimeout(()  => { }, 200); // eslint-disable-line

const groupByArray = (xs: any, key: any) => 
{ 
  return xs.reduce(function (rv: any, x: any) 
  { 
    const v: any = key instanceof Function ? key(x) : x[key];
    const el = rv.find((r:any) => r && r.key === v);
    if (el) 
    {
      el.values.push(x);
    } 
    else 
    { 
      rv.push({ key: v, values: [x] });
    } 
    return rv; 
  }, []); 
}
const group = <T, K extends keyof any>(list: T[], getKey: (item: T) => K) =>
list.reduce((previous, currentItem) => 
{
    const group = getKey(currentItem);
    if (!previous[group]) previous[group] = [];
    previous[group].push(currentItem);
    return previous;
}, {} as Record<K, T[]>);

const groupBy = (list: any, keyGetter: any) =>
{
    const map = new Map();
    list.forEach((item: any) => {
         const key = keyGetter(item);
         const collection = map.get(key);
         if (!collection) {
             map.set(key, [item]);
         } else {
             collection.push(item);
         }
    });
    return map;
}
export const date_str = (dt: string|null|undefined, format: DateFormat) =>
{
    if (dt == null || dt == undefined)
        return ""
    else
    {
        const d = DateTime.parse(dt);
        const date = d.to_string(format);
        return date;
    }
}
export const base64_to_uint8_array = (base64: string) =>  
{
  const binary = atob(base64);
  const len = binary.length;
  const bytes = new Uint8Array(len);
  for (let i = 0; i < len; i++)
      bytes[i] = binary.charCodeAt(i);
  return bytes;
}

export const component_visible = < T extends VNode<RendererNode, RendererElement, {[key: string]: any;}> >(vis: boolean, f:() => T) =>
{
  if (vis)
      return f();
  else return h('span')
}


  function isError (e: unknown)  : [string, string]
    {
      if(e instanceof Error)
      {
        console.log(e.name);
        return [e.name, e.message];
      }
      return ["", ""];
    }

    function deep_clone<T>(obj: T): T
    {
      return JSON.parse(JSON.stringify(obj)) as T;
    }

export interface CompressImageOptions 
{
  quality?: number // Качество изображения (0-1)
  maxWidth?: number // Максимальная ширина
  maxHeight?: number // Максимальная высота
  mimeType?: string // Тип выходного файла ('image/jpeg', 'image/png' и т.д.)
}

/**
 * Сжимает изображение с возможностью изменения размера
 * @param file Входной файл изображения
 * @param options Параметры сжатия
 * @returns Promise с Blob сжатого изображения
 */
export const compressImage = async (
  file: File,
  options: CompressImageOptions = {}
): Promise<Blob> => 
{
  const {
    quality = 0.8,
    maxWidth = 2048,
    maxHeight = 2048,
    mimeType = 'image/jpeg'
  } = options

  try 
  {
    const bitmap = await createImageBitmap(file)
    
    // Рассчитываем новые размеры с сохранением пропорций
    let width = bitmap.width
    let height = bitmap.height
    
    if (width > maxWidth || height > maxHeight) 
    {
      const ratio = Math.min(maxWidth / width, maxHeight / height)
      width = Math.floor(width * ratio)
      height = Math.floor(height * ratio)
    }

    const canvas = new OffscreenCanvas(width, height)
    const ctx = canvas.getContext('2d')
    
    if (!ctx) 
    {
      throw new Error('Could not get canvas context')
    }

    // Отрисовка с масштабированием
    ctx.drawImage(bitmap, 0, 0, width, height)
    
    // Конвертация в Blob с указанными параметрами
    const blob = await canvas.convertToBlob({
      type: mimeType,
      quality: Math.max(0, Math.min(1, quality)) // Обеспечиваем корректный диапазон
    })

    // Освобождаем ресурсы
    bitmap.close()
    
    return blob
  } 
  catch (error) 
  {
    console.error('Image compression failed:', error)
    // В случае ошибки возвращаем оригинальный файл
    return file
  }
}

function insert_into_sorted_array<T>(
  array: T[],
  element: T,
  compareFn: (a: T, b: T) => number
): T[] 
{
  if (array.length === 0) 
  {
    array.push(element);
    return array;
  }

  let low = 0;
  let high = array.length - 1;
  let index = array.length; // по умолчанию в конец

  while (low <= high) 
  {
    const mid = Math.floor((low + high) / 2);
    const comparison = compareFn(array[mid], element);

    if (comparison < 0) 
    {
      // array[mid] < element - ищем справа
      low = mid + 1;
    } 
    else if (comparison > 0) 
    {
      // array[mid] > element - ищем слева
      high = mid - 1;
      index = mid; // потенциальная позиция для вставки
    } 
    else 
    {
      // array[mid] == element - вставляем после
      index = mid + 1;
      break;
    }
  }

  // Если не нашли точное совпадение, используем индекс из бинарного поиска
  if (index === array.length) 
  {
    array.push(element);
  } 
  else 
  {
    array.splice(index, 0, element);
  }
  return array;
}

type LocalStorageFieldName = 'statistic' | 'dashboard-layout' | 'user' | 'statistic-status-options' | 'tasks-filter'
// Загрузка из localStorage
function load_from_localstorage<T>(field_name: LocalStorageFieldName): T | undefined
{
  const saved = localStorage.getItem(field_name);
  if (saved) 
  {
    const data: T = JSON.parse(saved);
    return data;
  }
};

// Сохранение в localStorage
function save_to_localstorage<T>(field_name: LocalStorageFieldName, value: T)
{
  localStorage.setItem(field_name, JSON.stringify(value));
};
function remove_from_localstorage<T>(field_name: LocalStorageFieldName)
{
  localStorage.removeItem(field_name);
};


class ColorHelper {
  static setOpacity(color: string, opacity: number): string {
    const normalizedOpacity = Math.max(0, Math.min(1, opacity));
    
    if (color.startsWith('#')) {
      return this.setHexOpacity(color, normalizedOpacity);
    } else if (color.startsWith('rgb(')) {
      return this.setRgbOpacity(color, normalizedOpacity);
    } else if (color.startsWith('rgba(')) {
      return this.updateRgbaOpacity(color, normalizedOpacity);
    } else {
      throw new Error(`Unsupported color format: ${color}`);
    }
  }
  
  private static setHexOpacity(hex: string, opacity: number): string {
    let cleanHex = hex.replace('#', '');
    
    if (cleanHex.length === 3) {
      cleanHex = cleanHex.split('').map(c => c + c).join('');
    }
    
    if (cleanHex.length !== 6) {
      throw new Error('Invalid HEX color format');
    }
    
    const alpha = Math.round(opacity * 255)
      .toString(16)
      .padStart(2, '0')
      .toUpperCase();
    
    return `#${cleanHex}${alpha}`;
  }
  
  private static setRgbOpacity(rgb: string, opacity: number): string {
    const matches = rgb.match(/rgb\((\d+),\s*(\d+),\s*(\d+)\)/);
    
    if (!matches) {
      throw new Error('Invalid RGB color format');
    }
    
    const [, r, g, b] = matches;
    return `rgba(${r}, ${g}, ${b}, ${opacity})`;
  }
  
  private static updateRgbaOpacity(rgba: string, opacity: number): string {
    const matches = rgba.match(/rgba\((\d+),\s*(\d+),\s*(\d+),\s*([\d.]+)\)/);
    
    if (!matches) {
      throw new Error('Invalid RGBA color format');
    }
    
    const [, r, g, b] = matches;
    return `rgba(${r}, ${g}, ${b}, ${opacity})`;
  }
}

export {sleepNow, timer, isError, ColorHelper, groupBy, deep_clone, group, insert_into_sorted_array, save_to_localstorage, load_from_localstorage, remove_from_localstorage}