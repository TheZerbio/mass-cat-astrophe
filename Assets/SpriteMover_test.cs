using UnityEngine;
using UnityEngine.InputSystem;

[RequireComponent(typeof(Rigidbody2D))]
public class SpriteMover : MonoBehaviour
{
    public float speed = 5f;
    private Rigidbody2D rb;
    private Vector2 input;

    void Awake() => rb = GetComponent<Rigidbody2D>();

    // Called automatically by PlayerInput for an action named "Move"
    void OnMove(InputValue value) => input = value.Get<Vector2>();

    void FixedUpdate() => rb.linearVelocity = input * speed;
}