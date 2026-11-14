using UnityEngine;
using UnityEngine.InputSystem;

public class WalkAnimator : MonoBehaviour
{
    public InputActionReference moveAction;
    [SerializeField] Vector2 startFacing = Vector2.down;

    Animator anim;
    Vector2 facing;

    static readonly int MoveX = Animator.StringToHash("MoveX");
    static readonly int MoveY = Animator.StringToHash("MoveY");
    static readonly int IsMoving = Animator.StringToHash("IsMoving");

    void Awake()
    {
        anim = GetComponent<Animator>();
        facing = startFacing;
        ApplyFacing();
    }

    void OnEnable()  => moveAction.action.Enable();
    void OnDisable() => moveAction.action.Disable();

    void Update()
    {
        Vector2 move = moveAction.action.ReadValue<Vector2>();
        bool moving = move.sqrMagnitude > 0.01f;

        anim.SetBool(IsMoving, moving);

        if (moving)
        {
            float ax = Mathf.Abs(move.x);
            float ay = Mathf.Abs(move.y);

            // Only change facing when one axis clearly dominates.
            // On a near-tie (diagonal), keep the current facing.
            if (ax > ay + 0.1f)
                facing = new Vector2(Mathf.Sign(move.x), 0);
            else if (ay > ax + 0.1f)
                facing = new Vector2(0, Mathf.Sign(move.y));

            ApplyFacing();
        }
    }

    void ApplyFacing()
    {
        anim.SetFloat(MoveX, facing.x);
        anim.SetFloat(MoveY, facing.y);
    }
}