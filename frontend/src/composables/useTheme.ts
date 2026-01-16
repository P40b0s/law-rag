import { darkTheme, lightTheme} from 'naive-ui';
import { ref, readonly } from 'vue';
const theme = ref(darkTheme);
export type Theme = 'dark' | 'light';
const current_theme = ref<Theme>('dark');
export const useTheme = () =>
{
    const dark_theme = () =>
    {
        theme.value = darkTheme;
        current_theme.value = 'dark';
    }
    const light_theme = () =>
    {
        theme.value = lightTheme;
        current_theme.value = 'light';
    }
    const get_current_theme = () =>
    {
        return readonly(current_theme)
    }
    return {dark_theme, light_theme, theme, get_current_theme}
}