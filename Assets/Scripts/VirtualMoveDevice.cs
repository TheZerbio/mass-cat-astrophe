using System.Runtime.InteropServices;
using UnityEngine;
using UnityEngine.InputSystem;
using UnityEngine.InputSystem.Controls;
using UnityEngine.InputSystem.Layouts;
using UnityEngine.InputSystem.LowLevel;
using UnityEngine.InputSystem.Utilities;
#if UNITY_EDITOR
using UnityEditor;
#endif

[StructLayout(LayoutKind.Sequential)]
public struct VirtualMoveState : IInputStateTypeInfo
{
    public FourCC format => new FourCC('V', 'M', 'O', 'V');

    [InputControl(layout = "Vector2")]
    public Vector2 move;
}

#if UNITY_EDITOR
[InitializeOnLoad]
#endif
[InputControlLayout(stateType = typeof(VirtualMoveState), displayName = "Virtual Move")]
public class VirtualMoveDevice : InputDevice
{
    public Vector2Control move { get; private set; }

    static VirtualMoveDevice()
    {
        InputSystem.RegisterLayout<VirtualMoveDevice>();
    }

    // Forces the static constructor to run in player builds
    [RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.BeforeSceneLoad)]
    static void Init() { }

    protected override void FinishSetup()
    {
        base.FinishSetup();
        move = GetChildControl<Vector2Control>("move");
    }
}