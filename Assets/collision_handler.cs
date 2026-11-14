using UnityEngine;

public class Destroyable : MonoBehaviour
{
    public bool isDestroyed = false;
    public Sprite destroyedSprite;   // assign in the Inspector
    public GameObject smokePrefab;

    private SpriteRenderer sr;
    private GameManager gm;

    void Awake()
    {
        gm = GameObject.Find("PersistantGameObjects").GetComponent<GameManager>();
            if(gm == null){
                Debug.Log("NO audio manager");
            }
        sr = GetComponent<SpriteRenderer>();
    }

    // Use this instead if the collider is NOT a trigger
    void OnCollisionEnter2D(Collision2D collision)
    {
        if (collision.gameObject.CompareTag("Player") && !isDestroyed)
        {
            Destroy_();
        }
    }
    
    void OnTriggerEnter2D(Collider2D other)
    {
        if (other.CompareTag("Player") && !isDestroyed)
        {
            Destroy_();
        }
    }

    void Destroy_()
    {
        isDestroyed = true;
        sr.sprite = destroyedSprite;
        gm.destructionScore += 67;
        gm.Sounds.CatSoundManager.PlayMeow();
        if (smokePrefab != null)
        {
            GameObject smoke = Instantiate(smokePrefab, transform.position, Quaternion.identity);
            Destroy(smoke, 3f);   // clean up after it finishes
        }
    }
}