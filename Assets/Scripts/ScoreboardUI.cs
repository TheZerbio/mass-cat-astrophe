using System.Collections.Generic;
using System.Linq;
using UnityEngine;
using UnityEngine.UI;

public class ScoreboardUI : MonoBehaviour
{
    [SerializeField] private RectTransform content;          // Scroll View/Viewport/Content
    [SerializeField] private ScoreboardEntry entryPrefab;
    [SerializeField] private ScrollRect scrollRect;

    private readonly List<ScoreboardEntry> _entries = new();
    private GameManager _gameManager;

    void Start()
    {
        _gameManager = GameObject.Find("PersistantGameObjects").GetComponent<GameManager>();
    }
    
    public struct PlayerScore
    {
        public string Name;
        public double Score;
        
        public PlayerScore(string name, double score)
        {
            Name = name;
            Score = score;
        }
    }

    public void Show(IEnumerable<PlayerScore> players)
    {
        var sorted = players.OrderByDescending(p => p.Score).ToList();

        // reuse existing rows, create missing ones, hide extras
        for (int i = 0; i < sorted.Count; i++)
        {
            if (i >= _entries.Count)
                _entries.Add(Instantiate(entryPrefab, content));

            _entries[i].gameObject.SetActive(true);
            _entries[i].Set(i + 1, sorted[i].Name, sorted[i].Score);
        }
        for (int i = sorted.Count; i < _entries.Count; i++)
            _entries[i].gameObject.SetActive(false);

        // force the layout to update now, then jump to the top
        LayoutRebuilder.ForceRebuildLayoutImmediate(content);
        scrollRect.verticalNormalizedPosition = 1f;
    }
}
