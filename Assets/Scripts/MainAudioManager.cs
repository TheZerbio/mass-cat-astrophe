using Assets.Scripts;
using UnityEngine;

public class MainAudioManager : MonoBehaviour
{
        public AudioSource BackgroundSource { get; private set; }
        public AudioSource CatMeowEffectSource {get; private set; }
        public AudioSource CatPurrSource { get; private set; }
        
        public AudioSource SfxSource { get; private set; }

        public SfxAudioManager SfxManager;
        public CatSoundManager CatSoundManager;
        public BackgroundAudioManager BackgroundAudioManager;
        
        [SerializeField] [Range(0,1)] private float musicVolume = 1.0f;
        public float MusicVolume
        {
            get => musicVolume;
            set { musicVolume = Mathf.Clamp01(value); ApplyVolume(); }
        }
        
        [SerializeField] [Range(0,1)] private float catVolume = 1.0f;
        public float CatVolume
        {
                get => catVolume;
                set { catVolume = Mathf.Clamp01(value); ApplyVolume(); }
        }
        
        [SerializeField] [Range(0,1)] private float sfxVolume = 1.0f;
        public float SfxVolume
        {
                get => sfxVolume;
                set { sfxVolume = Mathf.Clamp01(value); ApplyVolume(); }
        }
        
        [SerializeField] [Range(0,1)] private float volume = 1.0f;
        public float Volume
        {
                get => volume;
                set { volume = Mathf.Clamp01(value); ApplyVolume(); }
        }
        
        void Awake()
        {
                BackgroundSource = gameObject.AddComponent<AudioSource>();
                CatMeowEffectSource = gameObject.AddComponent<AudioSource>();
                CatPurrSource = gameObject.AddComponent<AudioSource>();
                SfxSource = gameObject.AddComponent<AudioSource>();
        }

        void Start()
        {
                ApplyVolume();
        }
        
        private void ApplyVolume()
        {
                // OnValidate can run on objects whose Awake never ran (prefab asset, deactivated duplicate)
                if (BackgroundSource == null) return;
                BackgroundSource.volume = volume*musicVolume;
                CatMeowEffectSource.volume = volume*catVolume;
                CatPurrSource.volume = volume*catVolume;
                SfxSource.volume = volume*sfxVolume;
        }
        
        void OnValidate()                                    // Inspector slider in Play Mode
        {
                if (Application.isPlaying) ApplyVolume();
        }
        
        //Pass trough
        void PlayMeowEffect()
        {
            
        }
}