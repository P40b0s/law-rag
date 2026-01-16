import { match } from "ts-pattern";
import { ref } from "vue";
import error_sound from '../../assets/mp3/error.mp3';
import ack_sound from '../../assets/mp3/ack.mp3';
type Sound = 'ack'| 'error';
export function useSound() 
{
  const is_playing = ref(false);
  
  const play_sound = (sound: Sound): void => 
  {
    if (is_playing.value) return;
    
    is_playing.value = true;
    const sound_path = match(sound)
    .with('ack', ()=> ack_sound)
    .with('error', ()=> error_sound)
    .exhaustive();
    const audio = new Audio(sound_path);
    
    audio.play().then(() => 
    {
        audio.onended = () => 
        {
            is_playing.value = false;
        };
    }).catch(error => 
    {
      console.error('Ошибка воспроизведения:', error);
      is_playing.value = false;
    });
  };
  
  return {
    play_sound,
    is_playing
  };
}