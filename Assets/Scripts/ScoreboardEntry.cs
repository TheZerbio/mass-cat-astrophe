using TMPro;
using UnityEngine;

public class ScoreboardEntry : MonoBehaviour
{
    [SerializeField] private TMP_Text rankText;
    [SerializeField] private TMP_Text nameText;
    [SerializeField] private TMP_Text scoreText;

    public void Set(int rank, string playerName, double score)
    {
        rankText.text  = $"#{rank}";
        nameText.text  = playerName;
        scoreText.text = score.ToString("F2");
    }
}