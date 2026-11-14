using UnityEngine;
using UnityEngine.InputSystem;

public class PlayerRotate : MonoBehaviour
{
    [Header("Input")]
    public InputActionReference moveAction;

    [Header("Rotation")]
    public float rotateSpeed = 720f;
    // -90 if your sprite faces UP by default, 0 if it faces RIGHT
    public float angleOffset = -90f;

    void OnEnable()
    {
        if (moveAction != null)
            moveAction.action.Enable();
    }

    void OnDisable()
    {
        if (moveAction != null)
            moveAction.action.Disable();
    }

    void Update()
    {
        if (moveAction == null) return;

        Vector2 move = moveAction.action.ReadValue<Vector2>();

        if (move.sqrMagnitude > 0.01f)
        {
            float angle = Mathf.Atan2(move.y, move.x) * Mathf.Rad2Deg + angleOffset;
            Quaternion target = Quaternion.Euler(0f, 0f, angle);
            transform.rotation = Quaternion.RotateTowards(
                transform.rotation, target, rotateSpeed * Time.deltaTime);
        }
    }
}