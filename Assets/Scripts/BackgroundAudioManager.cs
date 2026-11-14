using System;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.InputSystem;
using Random = UnityEngine.Random;

[RequireComponent(typeof(MainAudioManager))]
public class BackgroundAudioManager : MonoBehaviour
{
        
    [SerializeField] private string folder = "Sounds/Background";

    private MainAudioManager manager;
    private AudioSource source;
    private AudioClip[] clips;
    private Queue<AudioClip> queue = new Queue<AudioClip>();


    void Start()
    {
        
        manager = GetComponent<MainAudioManager>();
        source = manager.BackgroundSource;
        clips = Resources.LoadAll<AudioClip>(folder);
        if (clips.Length == 0)
            Debug.LogWarning($"No AudioClips found in Resources/{folder}");
        BuildQueue();
        
        source.Play();
    }

    private void FixedUpdate()
    {
        if (!source.isPlaying)
        {
            if (queue.Count > 0)
                PlayNext();                
            else
                BuildQueue();
            
        }
    }

    public void BuildQueue()
    {
        foreach (AudioClip clip in clips)
        {
            queue.Enqueue(clip);
        }
    }

    public void PlayNext()
    {
        source.clip = queue.Dequeue();
        source.Play();
    }
}