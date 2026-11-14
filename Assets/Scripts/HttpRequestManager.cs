using System;
using System.Collections;
using UnityEngine;
using UnityEngine.Networking;

public class HttpRequestManager
{
    [Serializable]
    public class NewGameResponse
    {
        public ulong game_id;   // must match the JSON key exactly
    }

    private const string BaseUrl = "https://cat-astrophe.pahlke.me";
    private readonly MonoBehaviour runner;   // runs the coroutines for this plain class

    public bool HasGameId { get; private set; }
    public ulong GameId { get; private set; }

    public event Action<ulong> OnGameCreated;
    public event Action<string> OnError;

    public HttpRequestManager(MonoBehaviour coroutineRunner)
    {
        runner = coroutineRunner;
    }

    public void CreateGame() => runner.StartCoroutine(CreateGameRoutine());

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
}