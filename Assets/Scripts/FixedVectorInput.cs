using System.Runtime.InteropServices;
using UnityEngine;
using UnityEngine.InputSystem;
using UnityEngine.InputSystem.Controls;
using UnityEngine.InputSystem.Layouts;
using UnityEngine.InputSystem.LowLevel;
using UnityEngine.InputSystem.Utilities;
using UnityEngine.InputSystem.Users;
public class FixedVectorInput : MonoBehaviour
{
    VirtualMoveDevice device;

    void OnEnable()
    {
        device = InputSystem.AddDevice<VirtualMoveDevice>();
        Debug.Log("Virtual device added: " + device);
        var playerInput = FindFirstObjectByType<PlayerInput>();
            if (playerInput != null){
                InputUser.PerformPairingWithDevice(device, playerInput.user);
            }
    }

    void OnDisable()
    {
        if (device != null) InputSystem.RemoveDevice(device);
    }

    void Update()
    {
        // Any Vector2 you want
        InputState.Change(device.move, new Vector2(0, 0));
    }
}
