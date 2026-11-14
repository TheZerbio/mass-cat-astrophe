using UnityEngine;
using UnityEngine.InputSystem;

public class WalkAnimator : MonoBehaviour
{
    public InputActionReference moveAction;
    Animator anim;

    void Awake()
    {
        anim = GetComponent<Animator>();
    }

    void Update()
    {
        Vector2 move = moveAction.action.ReadValue<Vector2>();
        anim.SetBool("IsMoving", move.sqrMagnitude > 0.01f);
    }
}