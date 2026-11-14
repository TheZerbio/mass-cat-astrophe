using UnityEngine;
using UnityEngine.UI;
using UnityEngine.Events;
using UnityEngine.InputSystem;

public class VoteCard : MonoBehaviour
{
    [SerializeField] Image[] options = new Image[3];
    [SerializeField] Image timerBar;              // optional: Image with Type = Filled
    [SerializeField] TMPro.TMP_Text timerText;    // optional: shows seconds remaining
    [SerializeField] Color normalColor = Color.white;
    [SerializeField] Color selectedColor = Color.yellow;
    [SerializeField] Color finalColor = Color.green;
    [SerializeField] float deadzone = 0.5f;
    [SerializeField] float scaleSelected = 1.15f;
    [SerializeField] float voteDuration = 10f;    // length of the voting window
    [SerializeField] InputActionReference moveAction;

    InputAction move => moveAction.action;
    public UnityEvent<int> onVoted;               // passes the final index (0-2)

    int index = -1;         // -1 = nothing selected yet
    bool hasInput;          // has the stick left the center at any point?
    float timeLeft;
    bool locked;



    

    void OnEnable()
    {
        move.Enable();
        StartVote();
    }

    void OnDisable() => move.Disable();

    public void StartVote()
    {
        timeLeft = voteDuration;
        locked = false;
        hasInput = false;
        index = -1;
        Refresh(selectedColor);
        UpdateTimerUI();
    }

    void Update()
    {
        if (locked) return;

        float x = move.ReadValue<Vector2>().x;
        int target = index;

        if (x < -deadzone)      { target = 0; hasInput = true; }
        else if (x > deadzone)  { target = 2; hasInput = true; }
        else if (hasInput && Mathf.Abs(x) < deadzone * 0.5f) target = 1;

        if (target != index)
        {
            index = target;
            Refresh(selectedColor);
        }

        timeLeft -= Time.deltaTime;
        UpdateTimerUI();

        if (timeLeft <= 0f)
        {
            timeLeft = 0f;
            locked = true;

            if (!hasInput)
            {
                index = Random.Range(0, options.Length);   // no vote: pick randomly
                Debug.Log("No input, random pick.");
            }

            Refresh(finalColor);
            onVoted?.Invoke(index);
            Debug.Log($"Voted for option {index}");
        }
    }

    void UpdateTimerUI()
    {
        if (timerBar) timerBar.fillAmount = timeLeft / voteDuration;
        if (timerText) timerText.text = Mathf.CeilToInt(timeLeft).ToString();
    }

    void Refresh(Color highlight)
    {
        for (int i = 0; i < options.Length; i++)
        {
            bool sel = i == index;
            options[i].color = sel ? highlight : normalColor;
            options[i].transform.localScale = Vector3.one * (sel ? scaleSelected : 1f);
        }
    }
}