using System.Collections;
using UnityEngine;

public class Destroyable : MonoBehaviour
{
    public bool isDestroyed = false;
    public Sprite destroyedSprite;
    public Sprite destroyedSprite2;
    public GameObject smokePrefab;

    [Header("Voting")]
    [SerializeField] float resultDelay = 1.5f;

    private SpriteRenderer sr;
    private GameManager gm;
    private bool voting;

    // Shared by every Destroyable, so the card is only searched for once
    static VoteCard cachedCard;

    // The player's movement script (found when the player touches us)
    private Behaviour frozenMovement;
    private Rigidbody2D frozenBody;

    void Awake()
    {
        GameObject persistent = GameObject.Find("PersistantGameObjects");
        if (persistent != null) gm = persistent.GetComponent<GameManager>();
        if (gm == null) Debug.Log("NO game manager");

        sr = GetComponent<SpriteRenderer>();
    }

    VoteCard GetCard()
    {
        if (cachedCard == null)   // also covers a destroyed card after a scene reload
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
    if (isDestroyed || voting || card == null) return;
    voting = true;

    FreezePlayer(playerCol);

    card.onVoted.RemoveListener(OnVoted);
    card.onVoted.AddListener(OnVoted);
    card.gameObject.SetActive(true);
}

RigidbodyConstraints2D savedConstraints;

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
        frozenBody.constraints = RigidbodyConstraints2D.FreezeAll;  // works whatever the movement script is
    }

    // Replace PlayerController with your movement script's name
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
        GetCard().onVoted.RemoveListener(OnVoted);
        StartCoroutine(FinishVote(choice));
    }

    IEnumerator FinishVote(int choice)
    {
        yield return new WaitForSeconds(resultDelay);

        GetCard().gameObject.SetActive(false);
        UnfreezePlayer();
        voting = false;

        Destroy_(choice);
    }

    // choice: 0 = left, 1 = middle, 2 = right
    void Destroy_(int choice)
    {
        switch (choice)
        {
            case 0: sr.sprite = destroyedSprite;  break;
            case 1: sr.sprite = destroyedSprite2; break;
            case 2: return;   // spared
        }

        isDestroyed = true;
        gm.destructionScore += 67;
        gm.Sounds.CatSoundManager.PlayMeow();

        if (smokePrefab != null)
        {
            GameObject smoke = Instantiate(smokePrefab, transform.position, Quaternion.identity);
            Destroy(smoke, 3f);
        }
    }
}