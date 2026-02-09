import { match } from "ts-pattern";

// Типы для работы с датой/временем бэкенда
export interface BackendDateTime {
    year: number;
    month: number;
    day: number;
    hour: number;
    minute: number;
    second: number;
}

export enum DateFormat {
    /** "2024-01-23T23:54:00" */
    SerializedDateTime,
    /** "24.05.1983 23:54:00" */
    DateTime,
    /** "2024-01-23" */
    SerializedDate,
    /** "23:54" без секунд */
    Time,
    /** "24.05.1983" */
    DotDate,
    CalendarFormat = 'dd.MM.yyyy',
    DateMonth = 'dd.MM',
}

export class DateTime {
    public year: number = 0;
    public month: number = 0;
    public day: number = 0;
    public hour: number = 0;
    public minute: number = 0;
    public second: number = 0;
    public weekDay: number = 0;
    public date: Date = new Date();

    private constructor(date: string | Date | number | BackendDateTime) {
        if (typeof date === "string") {
            this.parseString(date);
        } else if (typeof date === "number") {
            this.parseNumber(date);
        } else if (date instanceof Date) {
            this.parseDate(date);
        } else {
            // BackendDateTime object
            this.parseBackendDateTime(date);
        }
    }

    private parseString(date: string): void {
        const serialized_format_splitted = date.split("T");
        if (serialized_format_splitted.length > 1) {
            const datePart = serialized_format_splitted[0];
            const timePart = serialized_format_splitted[1];
            this.split_date(datePart, "-", [0, 1, 2]);
            this.split_time(timePart);
            this.date = new Date(this.year, this.month - 1, this.day, this.hour, this.minute, this.second);
            this.weekDay = this.date.getDay();
        } else if (date.includes(".")) {
            this.split_date(date, ".", [2, 1, 0]);
            this.date = new Date(this.year, this.month - 1, this.day, this.hour, this.minute, this.second);
            this.weekDay = this.date.getDay();
        } else if (date.includes("-")) {
            const split = date.split("-");
            if (split.length > 1) {
                const order = split[0].length === 4 ? [0, 1, 2] : [2, 1, 0];
                this.split_date(date, "-", order);
            }
            this.date = new Date(this.year, this.month - 1, this.day, this.hour, this.minute, this.second);
            this.weekDay = this.date.getDay();
        }
    }

    private parseNumber(timestamp: number): void {
        this.date = new Date(timestamp);
        this.updateFromDate();
    }

    private parseDate(date: Date): void {
        this.date = date;
        this.updateFromDate();
    }

    private parseBackendDateTime(dt: BackendDateTime): void {
        this.year = dt.year;
        this.month = dt.month;
        this.day = dt.day;
        this.hour = dt.hour;
        this.minute = dt.minute;
        this.second = dt.second;
        this.date = new Date(this.year, this.month - 1, this.day, this.hour, this.minute, this.second);
        this.weekDay = this.date.getDay();
    }

    private updateFromDate(): void {
        this.weekDay = this.date.getDay();
        this.year = this.date.getFullYear();
        this.month = this.date.getMonth() + 1;
        this.day = this.date.getDate();
        this.hour = this.date.getHours();
        this.minute = this.date.getMinutes();
        this.second = this.date.getSeconds();
    }

    static parse(date: string | Date | number | BackendDateTime): DateTime {
        return new DateTime(date);
    }

    static now(): DateTime {
        return new DateTime(new Date());
    }

    static today(): DateTime {
        const dt = new DateTime(new Date());
        return dt.set_hours(0, 0);
    }

    static yesterday(): DateTime {
        return DateTime.today().add_days(-1);
    }

    static tomorrow(): DateTime {
        return DateTime.today().add_days(1);
    }

    clone(): DateTime {
        return DateTime.parse(new Date(this.date));
    }

    set_date(date: number): DateTime {
        this.date.setDate(date);
        this.updateFromDate();
        return this;
    }

    set_hours(h: number, m: number, s: number = 0): DateTime {
        this.date.setHours(h, m, s, 0);
        this.updateFromDate();
        return this;
    }

    set_year(y: number): DateTime {
        this.date.setFullYear(y);
        this.updateFromDate();
        return this;
    }

    set_month(m: number): DateTime {
        this.date.setMonth(m - 1);
        this.updateFromDate();
        return this;
    }

    // Comparison methods
    gt(date: DateTime): boolean {
        return this.as_date() > date.as_date();
    }

    lt(date: DateTime): boolean {
        return this.as_date() < date.as_date();
    }

    eq(date: DateTime): boolean {
        return this.as_date().getTime() === date.as_date().getTime();
    }

    greater_or_equal_then(date: DateTime): boolean {
        return this.gt(date) || this.eq(date);
    }

    lower_or_equal_then(date: DateTime): boolean {
        return this.lt(date) || this.eq(date);
    }

    // Date checking methods
    isToday(): boolean {
        const today = DateTime.today();
        return this.year === today.year && this.month === today.month && this.day === today.day;
    }

    isYesterday(): boolean {
        const yesterday = DateTime.yesterday();
        return this.year === yesterday.year && this.month === yesterday.month && this.day === yesterday.day;
    }

    isTomorrow(): boolean {
        const tomorrow = DateTime.tomorrow();
        return this.year === tomorrow.year && this.month === tomorrow.month && this.day === tomorrow.day;
    }

    isSameDay(other: DateTime): boolean {
        return this.year === other.year && this.month === other.month && this.day === other.day;
    }

    isSameMonth(other: DateTime): boolean {
        return this.year === other.year && this.month === other.month;
    }

    isSameYear(other: DateTime): boolean {
        return this.year === other.year;
    }

    isWeekend(): boolean {
        return this.weekDay === 0 || this.weekDay === 6;
    }

    isWeekday(): boolean {
        return !this.isWeekend();
    }

    // Start/End methods
    startOfDay(): DateTime {
        return this.clone().set_hours(0, 0, 0);
    }

    endOfDay(): DateTime {
        return this.clone().set_hours(23, 59, 59);
    }

    startOfMonth(): DateTime {
        const dt = this.clone();
        dt.date.setDate(1);
        dt.updateFromDate();
        return dt.set_hours(0, 0, 0);
    }

    endOfMonth(): DateTime {
        const dt = this.clone();
        dt.date.setMonth(dt.date.getMonth() + 1, 0);
        dt.updateFromDate();
        return dt.set_hours(23, 59, 59);
    }

    startOfYear(): DateTime {
        const dt = this.clone();
        dt.date.setMonth(0, 1);
        dt.updateFromDate();
        return dt.set_hours(0, 0, 0);
    }

    endOfYear(): DateTime {
        const dt = this.clone();
        dt.date.setMonth(11, 31);
        dt.updateFromDate();
        return dt.set_hours(23, 59, 59);
    }

    as_date(): Date {
        return this.date;
    }

    // Для возможности использования операторов сравнения < = >
    valueOf(): number {
        return this.as_date().getTime();
    }

    // Arithmetic methods
    add_days(days: number): DateTime {
        this.date.setDate(this.date.getDate() + days);
        this.updateFromDate();
        return this;
    }

    add_months(months: number): DateTime {
        this.date.setMonth(this.date.getMonth() + months);
        this.updateFromDate();
        return this;
    }

    add_years(years: number): DateTime {
        this.date.setFullYear(this.date.getFullYear() + years);
        this.updateFromDate();
        return this;
    }

    add_hours(hours: number): DateTime {
        this.date.setHours(this.date.getHours() + hours);
        this.updateFromDate();
        return this;
    }

    add_minutes(minutes: number): DateTime {
        this.date.setMinutes(this.date.getMinutes() + minutes);
        this.updateFromDate();
        return this;
    }

    subtract_days(days: number): DateTime {
        return this.add_days(-days);
    }

    subtract_months(months: number): DateTime {
        return this.add_months(-months);
    }

    subtract_years(years: number): DateTime {
        return this.add_years(-years);
    }

    // Formatting methods
    to_string(format: DateFormat): string {
        const day = this.day < 10 ? "0" + this.day.toString() : this.day.toString();
        const month = this.month < 10 ? "0" + this.month.toString() : this.month.toString();
        const hour = this.hour < 10 ? "0" + this.hour.toString() : this.hour.toString();
        const minute = this.minute < 10 ? "0" + this.minute.toString() : this.minute.toString();
        const sec = this.second < 10 ? "0" + this.second.toString() : this.second.toString();

        return match(format)
            .returnType<string>()
            .with(DateFormat.SerializedDateTime, () => `${this.year}-${month}-${day}T${hour}:${minute}:${sec}`)
            .with(DateFormat.DateTime, () => `${day}.${month}.${this.year} ${hour}:${minute}:${sec}`)
            .with(DateFormat.SerializedDate, () => `${day}-${month}-${this.year}`)
            .with(DateFormat.DotDate, () => `${day}.${month}.${this.year}`)
            .with(DateFormat.Time, () => `${hour}:${minute}`)
            .with(DateFormat.CalendarFormat, () => `${day}.${month}.${this.year}`)
            .with(DateFormat.DateMonth, () => `${day}.${month}`)
            .otherwise(() => `${this.year}-${month}-${day}T${hour}:${minute}:${sec}`);
    }

    // Относительное форматирование
    toRelativeString(): string {
        const now = DateTime.now();
        const diffMs = now.as_date().getTime() - this.as_date().getTime();
        const diffSeconds = Math.floor(diffMs / 1000);
        const diffMinutes = Math.floor(diffSeconds / 60);
        const diffHours = Math.floor(diffMinutes / 60);
        const diffDays = Math.floor(diffHours / 24);

        if (diffSeconds < 60) {
            return 'только что';
        } else if (diffMinutes < 60) {
            return `${diffMinutes} ${getPluralForm(diffMinutes, 'минуту', 'минуты', 'минут')} назад`;
        } else if (diffHours < 24) {
            return `${diffHours} ${getPluralForm(diffHours, 'час', 'часа', 'часов')} назад`;
        } else if (diffDays === 1) {
            return 'вчера';
        } else if (diffDays < 7) {
            return `${diffDays} ${getPluralForm(diffDays, 'день', 'дня', 'дней')} назад`;
        } else {
            return this.to_string(DateFormat.DotDate);
        }
    }

    // Конвертация в формат бэкенда
    toBackendDateTime(): BackendDateTime {
        return {
            year: this.year,
            month: this.month,
            day: this.day,
            hour: this.hour,
            minute: this.minute,
            second: this.second
        };
    }

    /** год - 0 месяц - 1 день - 2 */
    private split_date(date: string, splitter: string, ord: number[]): void {
        const splitted_date = date.split(splitter).map(m => Number.parseInt(m));
        this.year = splitted_date[ord[0]];
        this.month = splitted_date[ord[1]];
        this.day = splitted_date[ord[2]];
    }

    private split_time(time: string): void {
        const splitted_time = time.split(":").map(m => Number.parseInt(m));
        if (splitted_time.length > 0) {
            this.hour = splitted_time[0] || 0;
            this.minute = splitted_time[1] || 0;
            this.second = splitted_time[2] || 0;
        }
    }

    days_between(end_date: DateTime): number {
        const ms_per_day = 1000 * 60 * 60 * 24;
        const diff = end_date.as_date().getTime() - this.as_date().getTime();
        return Math.floor(Math.abs(diff) / ms_per_day) + 1;
    }

    /**
     * Получение всех дат в искомом диапазоне
     */
    static between_dates_array(start_date: DateTime, end_date: DateTime): Date[] {
        const dates: Date[] = [];
        const current_date = new Date(start_date.as_date());
        current_date.setHours(0, 0, 0, 0);
        const end = new Date(end_date.as_date());
        end.setHours(0, 0, 0, 0);

        while (current_date <= end) {
            dates.push(new Date(current_date));
            current_date.setDate(current_date.getDate() + 1);
        }
        return dates;
    }

    between(date_from: DateTime, date_to: DateTime): boolean {
        const current = this.clone().set_hours(0, 0);
        return date_from <= current && date_to >= current;
    }

    calc_end_date(days_count: number = 1, format: DateFormat): string {
        const dc = this.date.getDate() + days_count;
        const ed = this.clone().set_date(dc);
        return ed.to_string(format);
    }
}

// Типы для прогресса
export type DateProgress = {
    /** Текущий процесс в процентах */
    progress: number;
    /** Количество в единицах сколько осталось */
    left: number;
    /** В единицах сколько между первой единицей и второй единицей */
    overall: number;
}

export type TimeProgress = {
    /** Текущий процесс в процентах */
    progress: number;
    /** Количество в минутах сколько осталось */
    minutes_left: number;
    /** Количество в часах сколько осталось */
    hours: number;
    minutes: number;
}

// Utility functions
export const dateToString = (date: Date): string => {
    return date.toLocaleString('ru-RU', {
        year: "numeric",
        month: '2-digit',
        day: '2-digit',
    });
}

export const timeToString = (date: Date): string => {
    return date.toLocaleString('ru-RU', {
        hour: '2-digit',
        minute: '2-digit',
    });
}

export const dateTimeToString = (date: Date): string => {
    return date.toLocaleString('ru-RU', {
        year: "numeric",
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
    });
}

export const getDaysDiff = (start_date: Date, end_date: Date): DateProgress => {
    const date_now = new Date().setHours(0, 0, 0);
    const oneDay = 24 * 60 * 60 * 1000;
    const diffFullVacation = Math.round(Math.abs((end_date.getTime() - start_date.getTime()) / oneDay)) + 1;
    const diffFromNow = Math.round(((end_date.getTime() - date_now) / oneDay)) + 1;

    return {
        progress: ((100 - Math.round((diffFromNow / diffFullVacation) * 100))),
        left: diffFromNow,
        overall: diffFullVacation
    };
}

export const timeLeft = (target_time: number | undefined): TimeProgress | undefined => {
    if (!target_time) return undefined;

    const date_now = new Date();
    const oneMinute = 60 * 1000;
    const diffFullVacation = Math.round(Math.abs((target_time - date_now.getTime()) / oneMinute)) + 1;
    const diffFromNow = Math.round(Math.abs((date_now.getTime() - date_now.setHours(0, 0, 0, 0)) / oneMinute)) + 1;
    const process = Math.trunc((diffFullVacation / diffFromNow) * 100);

    return {
        progress: process,
        minutes_left: diffFullVacation,
        hours: Math.trunc(diffFullVacation / 60),
        minutes: diffFullVacation % 60,
    };
}

export const to_rfc3339 = (date: Date): string => {
    const year = date.getFullYear();
    const month = (date.getMonth() + 1).toString().padStart(2, '0');
    const day = date.getDate().toString().padStart(2, '0');
    const hours = date.getHours().toString().padStart(2, '0');
    const minutes = date.getMinutes().toString().padStart(2, '0');
    const seconds = date.getSeconds().toString().padStart(2, '0');
    const milliseconds = date.getMilliseconds().toString().padStart(3, '0');

    return `${year}-${month}-${day}T${hours}:${minutes}:${seconds}.${milliseconds}Z`;
};

export const tuple_to_rfc3339 = (range: [number, number]): [string, string] => {
    const [a, b] = range;
    const from = new Date(a);
    const to_tmp = new Date(b);
    const to = new Date(to_tmp.getFullYear(), to_tmp.getMonth(), to_tmp.getDate(), 23, 59, 59, 999);
    return [to_rfc3339(from), to_rfc3339(to)];
}

// Конвертация между форматами бэкенда и фронтенда
export const backendDateTimeToDate = (dt: BackendDateTime): Date => {
    return new Date(dt.year, dt.month - 1, dt.day, dt.hour, dt.minute, dt.second);
}

export const dateToBackendDateTime = (date: Date): BackendDateTime => {
    return {
        year: date.getFullYear(),
        month: date.getMonth() + 1,
        day: date.getDate(),
        hour: date.getHours(),
        minute: date.getMinutes(),
        second: date.getSeconds()
    };
}

export const formatBackendDateTime = (dt: BackendDateTime): string => {
    const day = dt.day.toString().padStart(2, '0');
    const month = dt.month.toString().padStart(2, '0');
    return `${day}.${month}.${dt.year}`;
}

// Вспомогательная функция для склонения
function getPluralForm(n: number, form1: string, form2: string, form5: string): string {
    n = Math.abs(n) % 100;
    const n1 = n % 10;
    if (n > 10 && n < 20) return form5;
    if (n1 > 1 && n1 < 5) return form2;
    if (n1 === 1) return form1;
    return form5;
}
