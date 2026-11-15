using System.Collections;
using TMPro;
using UnityEngine;

[RequireComponent(typeof(CanvasGroup))]
public class CommentPopup : MonoBehaviour
{
    [SerializeField] TMP_Text text;
    [SerializeField] float fadeTime = 0.3f;
    [SerializeField] float showTime = 2.5f;

    CanvasGroup group;
    Coroutine routine;

    static CommentPopup instance;

    [RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.SubsystemRegistration)]
    static void ResetStatics() => instance = null;

    void Awake()
    {
        group = GetComponent<CanvasGroup>();
        group.alpha = 0f;
        if (!text) text = GetComponentInChildren<TMP_Text>();
    }

    // Call from anywhere: CommentPopup.Show("Meow!");
    public static void Show(string message)
    {
        if (string.IsNullOrEmpty(message)) return;

        if (instance == null)
            instance = FindFirstObjectByType<CommentPopup>(FindObjectsInactive.Include);
        if (instance == null) return;

        instance.Display(message);
    }

    void Display(string message)
    {
        gameObject.SetActive(true);
        text.text = message;
        if (routine != null) StopCoroutine(routine);
        routine = StartCoroutine(Run());
    }

    IEnumerator Run()
    {
        yield return Fade(0f, 1f);
        yield return new WaitForSecondsRealtime(showTime);
        yield return Fade(1f, 0f);
        routine = null;
    }

    IEnumerator Fade(float from, float to)
    {
        for (float t = 0f; t < fadeTime; t += Time.unscaledDeltaTime)
        {
            group.alpha = Mathf.Lerp(from, to, t / fadeTime);
            yield return null;
        }
        group.alpha = to;
    }
}