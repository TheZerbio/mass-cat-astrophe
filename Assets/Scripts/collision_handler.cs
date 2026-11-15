using System.Collections;
using UnityEngine;

public class Destroyable : MonoBehaviour
{
    public enum Outcome { Destroy, Spare }

    [System.Serializable]
    public class VoteChoice
    {
        public string label = "Option";
        public Sprite icon;                  // optional, shown on the card
        public Outcome outcome = Outcome.Destroy;
        public Sprite resultSprite;          // sprite to swap to if Destroy
        public int score = 67;
        [TextArea] public string comment;
    }

    public bool isDestroyed = false;

    [Header("Legacy sprites (fallback if a choice has no Result Sprite)")]
    public Sprite destroyedSprite;
    public Sprite destroyedSprite2;

    public GameObject smokePrefab;

    [Header("Voting")]
    [SerializeField] float resultDelay = 1.5f;
    [SerializeField] bool debugLogs = true;

    // index 0 = left, 1 = middle, 2 = right
    public VoteChoice[] choices = new VoteChoice[3]
    {
        new VoteChoice { label = "Destroy" },
        new VoteChoice { label = "Scratch", score = 30 },
        new VoteChoice { label = "Leave it", outcome = Outcome.Spare, score = 0 }
    };

    private SpriteRenderer sr;
    private GameManager gm;
    private bool voting;

    // Shared by every Destroyable, so the card is only searched for once
    static VoteCard cachedCard;

    private Behaviour frozenMovement;
    private Rigidbody2D frozenBody;
    private RigidbodyConstraints2D savedConstraints;

    [RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.SubsystemRegistration)]
    static void ResetStatics() => cachedCard = null;

    void Awake()
    {
        GameObject persistent = GameObject.Find("PersistantGameObjects");
        if (persistent != null) gm = persistent.GetComponent<GameManager>();
        if (gm == null) Debug.Log("NO game manager");

        sr = GetComponent<SpriteRenderer>();
    }

    VoteCard GetCard()
    {
        if (cachedCard == null)
            cachedCard = FindFirstObjectByType<VoteCard>(FindObjectsInactive.Include);
        return cachedCard;
    }

    void OnCollisionEnter2D(Collision2D collision)
    {
        if (collision.gameObject.CompareTag("Player"))
            StartVote(collision.collider);
    }

    void OnTriggerEnter2D(Collider2D other)
    {
        if (other.CompareTag("Player"))
            StartVote(other);
    }

    void StartVote(Collider2D playerCol)
    {
        VoteCard card = GetCard();

        if (debugLogs)
            Debug.Log($"[Destroyable] StartVote on {name}: card={(card ? card.name : "NULL")}, isDestroyed={isDestroyed}, voting={voting}, choices={(choices == null ? "NULL" : choices.Length.ToString())}");

        if (isDestroyed || voting || card == null) return;
        if (choices == null || choices.Length == 0)
        {
            Debug.LogWarning($"[Destroyable] {name} has no choices set. Fill them in on the prefab.");
            return;
        }

        voting = true;
        FreezePlayer(playerCol);

        // Push THIS object's options into the shared card
        string[] labels = new string[choices.Length];
        Sprite[] icons = new Sprite[choices.Length];
        for (int i = 0; i < choices.Length; i++)
        {
            if (choices[i] == null) continue;   // skip empty slots instead of crashing
            labels[i] = choices[i].label;
            icons[i] = choices[i].icon;
        }
        card.SetOptions(labels, icons);

        card.onVoted.RemoveListener(OnVoted);
        card.onVoted.AddListener(OnVoted);
        card.gameObject.SetActive(true);   // OnEnable -> StartVote()
    }

    void FreezePlayer(Collider2D playerCol)
    {
        // attachedRigidbody finds the rigidbody even if the collider is on a child
        frozenBody = playerCol.attachedRigidbody;
        Transform root = frozenBody ? frozenBody.transform : playerCol.transform.root;

        if (frozenBody)
        {
            frozenBody.linearVelocity = Vector2.zero;
            frozenBody.angularVelocity = 0f;
            savedConstraints = frozenBody.constraints;
            frozenBody.constraints = RigidbodyConstraints2D.FreezeAll;
        }

        frozenMovement = root.GetComponentInChildren<SpriteMover>();
        if (frozenMovement) frozenMovement.enabled = false;
    }

    void UnfreezePlayer()
    {
        if (frozenBody) frozenBody.constraints = savedConstraints;
        if (frozenMovement) frozenMovement.enabled = true;
        frozenMovement = null;
        frozenBody = null;
    }

    void OnVoted(int choice)
    {
        VoteCard card = GetCard();
        if (card) card.onVoted.RemoveListener(OnVoted);
        StartCoroutine(FinishVote(choice));
    }

    IEnumerator FinishVote(int choice)
{
    yield return new WaitForSeconds(resultDelay);

    VoteCard card = GetCard();
    if (card) card.gameObject.SetActive(false);
    UnfreezePlayer();
    voting = false;

    if (choices != null && choice >= 0 && choice < choices.Length && choices[choice] != null)
        CommentPopup.Show(choices[choice].comment);

    Destroy_(choice);
}

    // choice: 0 = left, 1 = middle, 2 = right
    void Destroy_(int choice)
    {
        if (choices == null || choice < 0 || choice >= choices.Length || choices[choice] == null) return;
        VoteChoice c = choices[choice];

        if (c.outcome == Outcome.Spare){
            isDestroyed = true;
            return;   // spared; set isDestroyed = true here if it should be final
            }

        // Use the choice's sprite, else fall back to the legacy ones
        Sprite result = c.resultSprite;
        if (result == null) result = (choice == 1) ? destroyedSprite2 : destroyedSprite;
        if (result != null && sr != null) sr.sprite = result;

        isDestroyed = true;

        if (gm != null)
        {
            gm.destructionScore += c.score;
            gm.Sounds.CatSoundManager.PlayMeow();
        }

        if (smokePrefab != null)
        {
            GameObject smoke = Instantiate(smokePrefab, transform.position, Quaternion.identity);
            Destroy(smoke, 3f);
        }
    }
}