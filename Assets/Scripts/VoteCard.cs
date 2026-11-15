using UnityEngine;
using UnityEngine.UI;
using UnityEngine.Events;
using UnityEngine.InputSystem;

public class VoteCard : MonoBehaviour
{
    [SerializeField] Image[] options = new Image[3];
    [SerializeField] TMPro.TMP_Text[] labels = new TMPro.TMP_Text[3];   // the Label under each option
    [SerializeField] Image[] icons = new Image[3];                       // optional icon Images
    [SerializeField] Image timerBar;              // optional: Image with Type = Filled
    [SerializeField] TMPro.TMP_Text timerText;    // optional: shows seconds remaining
    [SerializeField] Color normalColor = Color.white;
    [SerializeField] Color selectedColor = Color.yellow;
    [SerializeField] Color finalColor = Color.green;
    [SerializeField] float deadzone = 0.5f;
    [SerializeField] float scaleSelected = 1.15f;
    [SerializeField] float voteDuration = 10f;    // length of the voting window
    [SerializeField] InputActionReference moveAction;
    [SerializeField] RectTransform optionsContainer;

    InputAction move => moveAction.action;
    public UnityEvent<int> onVoted;               // passes the final index (0-2)

    int index = -1;         // -1 = nothing selected yet
    float timeLeft;
    bool locked;
    bool enabledByMe;

    public void SetOptions(string[] newLabels, Sprite[] newIcons = null)
    {
        for (int i = 0; i < options.Length; i++)
        {
            if (labels != null && i < labels.Length && labels[i] && i < newLabels.Length)
                labels[i].text = newLabels[i];

            if (icons != null && i < icons.Length && icons[i])
            {
                bool has = newIcons != null && i < newIcons.Length && newIcons[i] != null;
                icons[i].gameObject.SetActive(has);
                if (has) icons[i].sprite = newIcons[i];
            }
        }
    }

    void OnEnable()
    {
        enabledByMe = !move.enabled;
        if (enabledByMe) move.Enable();
        StartVote();
        RebuildLayout();
    }

    void OnDisable()
    {
        if (enabledByMe) move.Disable();
    }

    public void StartVote()
    {
        timeLeft = voteDuration;
        locked = false;
        index = -1;
        Refresh(selectedColor);
        UpdateTimerUI();
    }

    void Update()
    {
        if (locked) return;

        Vector2 stick = move.ReadValue<Vector2>();
        int target = index;

        // Whichever direction is pushed furthest wins, so diagonals don't flicker
        if (stick.y > deadzone && stick.y > Mathf.Abs(stick.x)) target = 1;   // up = middle
        else if (stick.x < -deadzone) target = 0;                             // left
        else if (stick.x > deadzone)  target = 2;                             // right
        // otherwise (stick at rest or pulled down): keep the current selection

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

            if (index < 0)
            {
                index = Random.Range(0, options.Length);   // no selection made: random pick
                Debug.Log("No selection, random pick.");
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

    void RebuildLayout()
    {
        if (!optionsContainer) return;
        Canvas.ForceUpdateCanvases();
        LayoutRebuilder.ForceRebuildLayoutImmediate(optionsContainer);
    }
}