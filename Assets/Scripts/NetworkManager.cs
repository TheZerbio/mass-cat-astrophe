using UnityEngine;
using UnityEngine.InputSystem;
using UnityEngine.InputSystem.LowLevel;
using UnityEngine.SceneManagement;

namespace Assets.Scripts
{
    public class NetworkManager : MonoBehaviour
    {
        private GUID guid;
        
        InputAction interactAction;
        // Start is called once before the first execution of Update after the MonoBehaviour is created
        void Start()
        {
            guid = GUID.Generate();
            interactAction = InputSystem.actions.FindAction("Interact");
            //interactAction.performed += ctx => { SceneManager.LoadScene("TestScene2");};
        }

        // Update is called once per frame
        void Update()
        {
  
        }

        void FixedUpdate()
        {
        }
    }
}
