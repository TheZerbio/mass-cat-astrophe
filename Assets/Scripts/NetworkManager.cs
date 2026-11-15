using System.Collections.Generic;
using UnityEngine;
using NativeWebSocket;
using UnityEngine.Serialization;

[RequireComponent(typeof(GameManager),typeof(FixedVectorInput))]
public class NetworkManager : MonoBehaviour
{
    private GUID _guid;
    private WebSocket _websocket;
    private HttpRequestManager _httpRequestManager;
    public bool GameReady { get; private set; }
    private bool _gameIdReady = false;
    private bool _quitting = false;
    private int _reconnectAttempts = 0;
    private float _nextConnectTime = 0f;
    private const float MaxReconnectDelay = 10f;
    public bool disableWebsocket = false;
    public bool enableWebsocketOutput = true;
    private readonly string _baseURL = WebConfig.WebsocketBaseURL;
    private GameManager _gameManager;
    private  FixedVectorInput _fixedVectorInput;
    public int ActiveInputs { get; private set; }

    [System.Serializable]
    private class WSPayload
    {
        public float x;
        public float y;
        public int num_inputs;

        public string toString()
        {
            return $"WSPayload: {{x:{x},y:{y},num_inputs:{num_inputs}}}";
        }
    }
    /* Read from Websocket like that
     * websocket.OnMessage += (bytes) =>
        {
            var message = System.Text.Encoding.UTF8.GetString(bytes);
            Debug.Log("Received: " + message);
        };
     */
    void Start()
    {
        _gameManager = GetComponent<GameManager>();
        _fixedVectorInput = GetComponent<FixedVectorInput>();
        
        _httpRequestManager = new HttpRequestManager(this);
        _httpRequestManager.CreateGame();
        
        //Configure Websocket Connection
        Application.runInBackground = true; // Recommended for WebGL
    }

    void Update()
    {
        if (!_gameIdReady && _httpRequestManager.HasGameId)
        {
            _gameManager.gameId = (int)_httpRequestManager.GameId;
            _gameIdReady = true;
            Debug.Log("Game ID: " + _gameManager.gameId);
        }

        // (Re)connect whenever there is no socket and the backoff delay has passed
        if (_gameIdReady && _websocket == null && !_quitting && !disableWebsocket
            && Time.unscaledTime >= _nextConnectTime)
        {
            ConnectWebsocket();
        }
    }

    void ConnectWebsocket()
    {
        var url = $"{_baseURL}/game/{_gameManager.gameId}/ws/controls";
        if (_reconnectAttempts == 0)
            Debug.Log("Connecting to: " + url);
        else
            Debug.Log($"Reconnecting to: {url} (attempt {_reconnectAttempts})");

        var ws = new WebSocket(url);
        _websocket = ws;

        // Every handler checks "ws == _websocket" so events from an old, replaced socket are ignored
        ws.OnOpen += () =>
        {
            if (ws != _websocket) return;
            Debug.Log("Connection open!");
            _reconnectAttempts = 0;
            GameReady = true;
            SendWebSocketMessage("Hi From Unity :)");
        };
        ws.OnError += (e) =>
        {
            if (ws != _websocket) return;
            Debug.LogError("WebSocket error: " + e);
        };
        ws.OnClose += (code) =>
        {
            if (ws != _websocket) return;
            OnWebsocketClosed(code);
        };
        ws.OnMessage += (bytes) =>
        {
            if (ws != _websocket) return;
            var message = System.Text.Encoding.UTF8.GetString(bytes);
            WSPayload payload;
            try
            {
                payload = JsonUtility.FromJson<WSPayload>(message);
            }
            catch (System.Exception)
            {
                Debug.LogWarning("Ignoring non-JSON websocket message: " + message);
                return;
            }
            if (payload == null) return;
            if (enableWebsocketOutput)
                Debug.Log(payload.toString());
            ActiveInputs = payload.num_inputs;
            _fixedVectorInput.movement = new Vector2(payload.x, payload.y);
        };

        // Connect() only completes when the connection ends; errors are reported through OnError/OnClose
        _ = ws.Connect();
    }

    void OnWebsocketClosed(WebSocketCloseCode code)
    {
        _websocket = null;
        GameReady = false;
        ActiveInputs = 0;
        _fixedVectorInput.movement = Vector2.zero; // don't let the cat keep running on the last input

        if (_quitting) return;

        // 1s, 2s, 4s, 8s, then every 10s
        float delay = Mathf.Min(MaxReconnectDelay, Mathf.Pow(2, _reconnectAttempts));
        _reconnectAttempts++;
        _nextConnectTime = Time.unscaledTime + delay;

        // Abnormal (1006) = connection cut without a close frame (proxy timeout, server task died, network)
        Debug.LogWarning($"Connection closed: {code} ({(int)code}). Reconnecting in {delay:0}s");
    }

    async void SendWebSocketMessage(string message)
    {
        if (_websocket != null && _websocket.State == WebSocketState.Open)
        {
            await _websocket.SendText(message);
        }
        else
        {
            Debug.LogError("Tried to send Message but WebSocket is not open!");
        }
    }

    private async void OnApplicationQuit()
    {
        _quitting = true;
        _httpRequestManager?.StopGame();
        if (_websocket != null)
            await _websocket.Close();
    }

    public void StartGame()
    {
        _httpRequestManager.StartGame();
    }

    public List<ScoreboardUI.PlayerScore> GetPlayerScores()
    {
        return _httpRequestManager.GetPlayerScores();
    }
    
}