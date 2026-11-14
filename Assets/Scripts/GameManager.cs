using UnityEngine;

public class GameManager : MonoBehaviour
{
        //There will be exactly one instance of this Class that get initialized in the main menu/start scree -> Persists over all Scenes
        public double destructionScore = 0.0;
        
        /*
         * Exec Order Awake -> Start (once each)
         * Update() (every Frame)
         * FixedUpdate (Physics sim tick, use for collisions and Movement tyoe stuff)
         */
        void Awake()
        {
                
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