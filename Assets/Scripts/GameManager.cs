using QRCoder;
using UnityEngine;

[RequireComponent(typeof(MainAudioManager),typeof(NetworkManager))]
public class GameManager : MonoBehaviour
{
        private static int _managerId = 0;
        //There will be exactly one instance of this Class that get initialized in the main menu/start scree -> Persists over all Scenes
        // How to access from other scripts
        // private GameManager gm = GameObject.Find("PersistantGameObjects").GetComponent<GameManager>();
        // Check for availability/null condition afterwards, Test Scenes require the PersistantGameObjects prefab to be placed
        public double destructionScore = 0.0;
        public MainAudioManager Sounds;
        public NetworkManager Network;
        private int ownId;
        
        [System.NonSerialized] public int? gameId = null;
        
        /*
         * Exec Order Awake -> Start (once each)
         * Update() (every Frame)
         * FixedUpdate (Physics sim tick, use for collisions and Movement tyoe stuff)
         */
        void Awake()
        {
                ownId = _managerId++;
                if (ownId != 0)
                {
                        Destroy(gameObject);
                }
                Sounds=gameObject.GetComponent<MainAudioManager>();
                Network=gameObject.GetComponent<NetworkManager>();
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

        /// <summary>
        /// Generates the Join QR Code based on the Network as long as the Game is ready to start
        /// </summary>
        /// <param name="size"></param>
        /// <returns>Square QR Code Texture with 50px * size</returns>
        public Texture2D GenerateJoinQrCode(int size = 1)
        {
                var url = $"{WebConfig.HttpsBaseURL}/game/play?game_id={gameId.GetValueOrDefault(0)}";
                return QRCodeGenerator
                        .GenerateQrCode(url, QRCodeGenerator.ECCLevel.M).ToTexture2D(
                                false, (qrSize) =>
                                {
                                        // QR code will be magnified 5x times. So, if the original size (qrSize) was 50x50, the resulting Texture2D will be 250x250.
                                        // If you'll be displaying the QR code on a mesh or UI, there is no need to magnify.
                                        return size;
                                });
        }
}