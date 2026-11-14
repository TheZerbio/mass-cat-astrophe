using UnityEngine.InputSystem;
using UnityEngine.SceneManagement;

namespace Assets.Scripts
{
    using UnityEngine;

    [RequireComponent(typeof(MainAudioManager))]
    public class CatSoundManager : MonoBehaviour
    {
        [SerializeField] private string folder = "Sounds/Meows"; // path inside Assets/Resources/
        [SerializeField] private bool avoidRepeat = true;

        private MainAudioManager manager;
        private AudioSource meowSource;
        private AudioSource purrSource;
        private AudioClip[] clips;
        
        [SerializeField] AudioClip purrClip;
        
        private float purrDuration = 0.0f;
        private int lastIndex = -1;
        private InputAction interaction;
        

        void Start()
        {
            manager =  GetComponent<MainAudioManager>();
            meowSource = manager.CatMeowEffectSource;
            purrSource = manager.CatPurrSource;
            
            clips = Resources.LoadAll<AudioClip>(folder);

            if (clips.Length == 0)
                Debug.LogWarning($"No AudioClips found in Resources/{folder}");
            if (purrClip == null)
                Debug.LogWarning($"Purr Clip is missing");
            
            InitPurrSource();
            
            //Register
            manager.CatSoundManager = this;
            
            //Testing:
            interaction = InputSystem.actions.FindAction("Interact");
            interaction.started += _ => PlayMeow();
        }

        void FixedUpdate()
        {
            //Update Purr timer
            if (purrDuration > 0.0f && purrSource.isPlaying)
            {
                purrDuration -= Time.fixedDeltaTime;
                if (purrDuration <= 0.0f)
                {
                    purrDuration = 0.0f;
                    purrSource.Stop();
                } 
            }
            if (purrDuration > 0.0f && !purrSource.isPlaying)
                purrSource.Play();
        }

        public void PlayMeow()
        {
            if (clips.Length == 0) return;

            int index = Random.Range(0, clips.Length);
            if (avoidRepeat && clips.Length > 1)
            {
                while (index == lastIndex)
                    index = Random.Range(0, clips.Length);
            }
            lastIndex = index;

            meowSource.PlayOneShot(clips[index]);
        }

        private void InitPurrSource()
        {
            purrSource.clip = purrClip;
            meowSource.loop = true;
            
        }
        
        public void PlayPurr(float duration)
        {
            purrDuration = Mathf.Max(duration, purrDuration);
        }
    }
}