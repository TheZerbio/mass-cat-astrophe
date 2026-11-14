using System.Runtime.InteropServices;
using UnityEngine;
using UnityEngine.InputSystem;
using UnityEngine.InputSystem.Controls;
using UnityEngine.InputSystem.Layouts;
using UnityEngine.InputSystem.LowLevel;
using UnityEngine.InputSystem.Utilities;
public class FixedVectorInput : MonoBehaviour
{
    VirtualMoveDevice device;

    void OnEnable()
    {
        device = InputSystem.AddDevice<VirtualMoveDevice>();
         InputState.Change(device.move, new Vector2(1, 0));
    }

    void OnDisable()
    {
        if (device != null) InputSystem.RemoveDevice(device);
    }

    void Update()
    {
        // Any Vector2 you want
        InputState.Change(device.move, new Vector2(1, 0));
    }
}
