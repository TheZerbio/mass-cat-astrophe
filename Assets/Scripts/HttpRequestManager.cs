using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using UnityEngine;
using UnityEngine.Networking;

public class HttpRequestManager
{
    [Serializable]
    public class NewGameResponse
    {
        public ulong game_id;   // must match the JSON key exactly
    }

    private readonly string _baseUrl = WebConfig.HttpsBaseURL;
    private readonly MonoBehaviour runner;   // runs the coroutines for this plain class
    private List<ScoreboardUI.PlayerScore> ScoresBuffer = new List<ScoreboardUI.PlayerScore>();

    public bool HasGameId { get; private set; }
    public ulong GameId { get; private set; }

    public event Action<ulong> OnGameCreated;
    public event Action<string> OnError;

    public HttpRequestManager(MonoBehaviour coroutineRunner)
    {
        runner = coroutineRunner;
    }

    public void CreateGame() => runner.StartCoroutine(CreateGameRoutine());
    public void StopGame() => runner.StartCoroutine(StopGameRoutine());
    public void StartGame() => runner.StartCoroutine(StartGameRoutine());
    /// <summary>
    /// Requests the scores from the server. The request takes a moment, so the result can't be returned
    /// directly: onLoaded is called with the list once the answer has arrived (not called on errors).
    /// </summary>
    public void GetPlayerScores(Action<List<ScoreboardUI.PlayerScore>> onLoaded)
    {
        runner.StartCoroutine(GetScoresRoutine(onLoaded));
    }

    private IEnumerator CreateGameRoutine()
    {
        HasGameId = false;

        using var request = UnityWebRequest.Post($"{_baseUrl}/game/new", "{}", "application/json");
        yield return request.SendWebRequest();

        if (request.result != UnityWebRequest.Result.Success)
        {
            string msg = $"CreateGame failed: {request.responseCode} {request.error}\n{request.downloadHandler.text}";
            Debug.LogError(msg);
            OnError?.Invoke(msg);
            yield break;
        }

        GameId = JsonUtility.FromJson<NewGameResponse>(request.downloadHandler.text).game_id;
        HasGameId = true;
        OnGameCreated?.Invoke(GameId);
    }
    
    private IEnumerator StopGameRoutine()
    {
        if (HasGameId)
        {
            using var request = UnityWebRequest.Post($"{_baseUrl}/game/{GameId}/close", "{}", "application/json");
            yield return request.SendWebRequest();

            if (request.result != UnityWebRequest.Result.Success)
            {
                string msg = $"CloseGame failed: {request.responseCode} {request.error}\n{request.downloadHandler.text}";
                Debug.LogError(msg);
                OnError?.Invoke(msg);
                yield break;
            }
        }
    }
    
    private IEnumerator StartGameRoutine()
    {
        if (HasGameId)
        {
            using var request = UnityWebRequest.Post($"{_baseUrl}/game/{GameId}/start", "{}", "application/json");
            yield return request.SendWebRequest();

            if (request.result != UnityWebRequest.Result.Success)
            {
                string msg = $"StartGame failed: {request.responseCode} {request.error}\n{request.downloadHandler.text}";
                Debug.LogError(msg);
                OnError?.Invoke(msg);
                yield break;
            }
        }
    }
    
    private IEnumerator GetScoresRoutine(Action<List<ScoreboardUI.PlayerScore>> onLoaded)
    {
        if (!HasGameId)
        {
            Debug.LogWarning("GetScores: no game id yet, nothing to load");
            yield break;
        }

        using var request = UnityWebRequest.Get($"{_baseUrl}/game/{GameId}/statistics");
        yield return request.SendWebRequest();

        if (request.result != UnityWebRequest.Result.Success)
        {
            string msg = $"GetScores failed: {request.responseCode} {request.error}\n{request.downloadHandler.text}";
            Debug.LogError(msg);
            OnError?.Invoke(msg);
            yield break;
        }

        var response = JsonUtility.FromJson<ScoreBoardEntryData.HttpReturn>(request.downloadHandler.text);
        Debug.LogWarning("Recieved Scores:\n"+request.downloadHandler.text);
        var formated = new List<ScoreboardUI.PlayerScore>();
        if (response?.players != null)
        {
            foreach (var playerObject in response.players)
            {
                double score = 0;
                if (playerObject.similarities_agg != null)
                {
                    for (int i = 0; i < playerObject.similarities_agg.Length; i++)
                    {
                        score += playerObject.similarities_agg[i];
                    }
                    score /= playerObject.similarities_agg.Length;
                }
                formated.Add(new ScoreboardUI.PlayerScore(playerObject.name, score));
            }
        }
        else
        {
            Debug.LogWarning("GetScores: no players in response: " + request.downloadHandler.text);
        }

        ScoresBuffer = formated;
        onLoaded?.Invoke(formated);
    }
}