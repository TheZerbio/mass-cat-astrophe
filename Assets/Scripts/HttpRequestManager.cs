using System;
using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.Networking;

public class HttpRequestManager
{
    [Serializable]
    public class NewGameResponse
    {
        public ulong game_id;   // must match the JSON key exactly
    }

    private string BaseUrl = WebConfig.HttpsBaseURL;
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
    public List<ScoreboardUI.PlayerScore> GetPlayerScores()
    {
        runner.StartCoroutine(GetScoresRoutine());
        return ScoresBuffer;
    }

    private IEnumerator CreateGameRoutine()
    {
        HasGameId = false;

        using var request = UnityWebRequest.Post($"{BaseUrl}/game/new", "{}", "application/json");
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
            using var request = UnityWebRequest.Post($"{BaseUrl}/game/{GameId}/close", "{}", "application/json");
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
            using var request = UnityWebRequest.Post($"{BaseUrl}/game/{GameId}/start", "{}", "application/json");
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
    
    private IEnumerator GetScoresRoutine()
    {
        if (HasGameId)
        {
            using var request = UnityWebRequest.Get($"{BaseUrl}/game/{GameId}/statistics");
            yield return request.SendWebRequest();

            if (request.result != UnityWebRequest.Result.Success)
            {
                string msg = $"StartGame failed: {request.responseCode} {request.error}\n{request.downloadHandler.text}";
                Debug.LogError(msg);
                OnError?.Invoke(msg);
                yield break;
            }

            if (request.result == UnityWebRequest.Result.Success)
            {
                Debug.Log(request.downloadHandler.text);
                ScoresBuffer = JsonUtility.FromJson<List<ScoreboardUI.PlayerScore>>(request.downloadHandler.text);
            }
        }
    }
}