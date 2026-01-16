import { dateToString } from "@/services/date";
import { uuidv7, V7Generator } from "uuidv7";
import { type Ref, ref } from "vue";

export class TimeWarning
{
    _notifyBeforeTargetTime: number;
    _time?: string;
    constructor(text: string, time?: string, weekDay?: number[], date?: Date, notifyBeforeTargetTime: number = 30)
    {
        this.id = uuidv7();
        this._notifyBeforeTargetTime = notifyBeforeTargetTime;
        this.time = time;
        this.date = date;
        this.text = text;
        this.weekDay = weekDay;
    }
    id: string;
    weekDay?: number[];
    date?: Date;
    get time(): string|undefined
    {
        return this._time
    }    
    set time(t: string|undefined)
    {
        if(t)
        {
            const warn_time = t.split(":");
            this.warningTime = new Date().setHours((parseInt(warn_time[0])), (parseInt(warn_time[1])), 0, 0)
            this.startNotifyTime = new Date().setHours((parseInt(warn_time[0])), (parseInt(warn_time[1]) - this._notifyBeforeTargetTime), 0, 0)
            console.log(t);
            this._time = t;
        }
    }
    warningTime?: number;
    startNotifyTime?: number;
    text: string = "";
    showNotify: boolean = true;
    public isVisible: boolean = false;
    //методы класса не клонируются как положено, вроде изза того что надо делать Object.Assign потому что клонирование идет через JSON сериализацию
    // progress(): TimeProgress
    // {
    //     const t = timeLeft(this.warningTime);
    //     if(t)
    //         return t
    //     else
    //         return {progress: 0, minutes_left: 0, hours: 0, minutes: 0}
    // }
    //minutesLeft:() => number;
}

export const time_warnings = ref<TimeWarning[]>
([
    new TimeWarning("Тестовое предупреждение на определенную дату 1", "07:20", [1,2, 3, 4, 5], new Date(2025, 10, 9)),
    new TimeWarning("Тестовое предупреждение на определенную дату 1", "17:40", [1,2], new Date(2023, 11, 28)),
    new TimeWarning("Отправить факс тест COVID ", "10:00", [0,2]),
    new TimeWarning("Отзвонить дежурному УИС", "06:10"),
    new TimeWarning("Отзвонить дежурному Спецсвязи - 239-29", "06:15"),
    new TimeWarning("Проверка кнопки - 224-88", "07:45"),
    new TimeWarning("Передать записку  239-29", "09:10"),
    new TimeWarning("Передать эпидемиологическую справку", "17:00", [1,2,3,4]),
    new TimeWarning("Передать эпидемиологическую справку", "16:00", [5,6,0]),
    new TimeWarning("Передать  противопожарный акт", "16:00", [5]),
    new TimeWarning("Тестовое предупреждение на определенную дату 1", "15:25", [1,2,3,4,5],),
    new TimeWarning("Тестовое предупреждение на определенную дату 1", "01:30", [1,2,3,4,5,6,],),
    new TimeWarning("Тестовое предупреждение на определенную дату 1", "12:15", [1,2,3,4,5,6,],),
    new TimeWarning("Тестовое предупреждение на определенную дату 2", undefined, undefined, new Date(2023, 11, 29),),
    
]) as Ref<TimeWarning[]>

export const updateTimeWarnings = (tw: TimeWarning[]) =>
{
    time_warnings.value = sortTimeWarnings(tw);
}



const sorted_time_warnings = () =>
{

}
export const sortTimeWarnings = (tw: TimeWarning[]): TimeWarning[] =>
{
    tw.sort((a, b)=> 
    {
        if(a.startNotifyTime == undefined && b.startNotifyTime == undefined)
            return -1;
        if(a.startNotifyTime == undefined)
            return -1;
        if(b.startNotifyTime == undefined)
            return -1;
        //if(new Date().setHours(parseInt(a.time.split(":")[0]), parseInt(a.time.split(":")[1])) > new Date().setHours(parseInt(b.time.split(":")[0]), parseInt(b.time.split(":")[1])))
        if(a.startNotifyTime > b.startNotifyTime)
            return 1;
        else
            return -1;
    });
    return tw;
}
interface Timer
{
    current_date: Date,
    string_date: string,
    string_time: string,
    week_day: number,
    is_midnight: boolean,
    date_without_time: number
}
const d = new Date();
const timer = ref<Timer>(
    {
        current_date: d,
        string_date: dateToString(d),
        string_time: d.getHours() + ":" + d.getMinutes(),
        week_day: d.getDay(),
        is_midnight: d.getHours() == 0 && d.getMinutes() == 0,
        date_without_time: d.setHours(0, 0, 0, 0)
    }
)
const start_timer = () =>
{
    let intervalId = setInterval(() => 
    {
        const d = new Date();
        timer.value.current_date = d;
        timer.value.string_date = dateToString(d);
        timer.value.string_time = d.getHours() + ":" + d.getMinutes();
        timer.value.week_day = d.getDay();
        timer.value.is_midnight = d.getHours() == 0 && d.getMinutes() == 0;
        timer.value.date_without_time = d.setHours(0, 0, 0, 0);
        
        console.log("таймер обновлен", timer.value);
    }, 60000);
}

export const useTime = () =>
{

    return {start_timer, timer}
}

