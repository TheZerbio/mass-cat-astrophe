using UnityEngine;
using UnityEngine.InputSystem;
using UnityEngine.InputSystem.LowLevel;
using UnityEngine.InputSystem.Users;
using UnityEngine.SceneManagement;

public class FixedVectorInput : MonoBehaviour
{
    VirtualMoveDevice device;
    public Vector2 movement = new Vector2(0, 0);

    void OnEnable()
    {
        // A duplicate PersistantGameObjects (placed in a test scene) must not create its own device
        if (GameManager.Instance != null && GameManager.Instance.gameObject != gameObject) return;

        device = InputSystem.AddDevice<VirtualMoveDevice>();
        Debug.Log("Virtual device added: " + device);
        SceneManager.sceneLoaded += OnSceneLoaded;
        PairWithPlayer();
    }

    void OnDisable()
    {
        SceneManager.sceneLoaded -= OnSceneLoaded;
        if (device != null) InputSystem.RemoveDevice(device);
        device = null;
    }

    void OnSceneLoaded(Scene scene, LoadSceneMode mode) => PairWithPlayer();

    // The PlayerInput lives in the game scene, so pairing has to happen again whenever a scene loads
    void PairWithPlayer()
    {
        if (device == null) return;
        var playerInput = FindAnyObjectByType<PlayerInput>();
        if (playerInput != null)
        {
            InputUser.PerformPairingWithDevice(device, playerInput.user);
        }
    }

    void FixedUpdate()
    {
        if (device == null) return;
        InputState.Change(device.move, movement);
    }
}
