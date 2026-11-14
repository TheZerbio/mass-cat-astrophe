using UnityEngine;

[RequireComponent(typeof(MainAudioManager))]
public class SfxAudioManager : MonoBehaviour
{
        private MainAudioManager manager;
        private AudioSource source;
        
        [SerializeField] private AudioClip swooshSound;

        void Start()
        {
                manager = GetComponent<MainAudioManager>();
                source = GetComponent<AudioSource>();
                manager.SfxManager = this;
        }

        [ContextMenu("PlaySwooch")]
        public void PlaySwoosh()
        {
                source.PlayOneShot(swooshSound);
        }
}