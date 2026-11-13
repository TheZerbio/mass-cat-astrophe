using UnityEngine.InputSystem;

namespace Assets.Scripts
{
    using UnityEngine;

    [RequireComponent(typeof(AudioSource))]
    public class RandomSoundPlayer : MonoBehaviour
    {
        [SerializeField] private string folder = "Sounds/Meows"; // path inside Assets/Resources/
        [SerializeField] private bool avoidRepeat = true;

        private AudioSource source;
        private AudioClip[] clips;
        private int lastIndex = -1;
        private InputAction interaction;

        void Awake()
        {
            interaction = InputSystem.actions.FindAction("Interact");
            interaction.started += _ => PlayRandom();
            source = GetComponent<AudioSource>();
            clips = Resources.LoadAll<AudioClip>(folder);

            if (clips.Length == 0)
                Debug.LogWarning($"No AudioClips found in Resources/{folder}");
        }

        public void PlayRandom()
        {
            if (clips.Length == 0) return;

            int index = Random.Range(0, clips.Length);
            if (avoidRepeat && clips.Length > 1)
            {
                while (index == lastIndex)
                    index = Random.Range(0, clips.Length);
            }
            lastIndex = index;

            source.PlayOneShot(clips[index]);
        }
    }
}