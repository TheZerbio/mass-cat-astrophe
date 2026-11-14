using UnityEngine;

[RequireComponent(typeof(MainAudioManager))]
public class GameManager : MonoBehaviour
{
        //There will be exactly one instance of this Class that get initialized in the main menu/start scree -> Persists over all Scenes
        // How to access from other scripts
        // private GameManager gm = GameObject.Find("PersistantGameObjects").GetComponent<GameManager>();
        // Check for availability/null condition afterwards, Test Scenes require the PersistantGameObjects prefab to be placed
        public double destructionScore = 0.0;
        public MainAudioManager Sounds;
        
        /*
         * Exec Order Awake -> Start (once each)
         * Update() (every Frame)
         * FixedUpdate (Physics sim tick, use for collisions and Movement tyoe stuff)
         */
        void Awake()
        {
                Sounds=gameObject.GetComponent<MainAudioManager>();
        }
        void Start()
        {
                
        }

        void Update()
        {
                
        }

        void FixedUpdate()
        {
                
        }
}