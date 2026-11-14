using UnityEngine;
using NativeWebSocket;
using UnityEngine.Serialization;

[RequireComponent(typeof(GameManager))]
public class NetworkManager : MonoBehaviour
{
    private GUID _guid;
    private WebSocket _websocket;
    private HttpRequestManager _httpRequestManager;
    public bool GameReady { get; private set; }
    private bool _gameIdReady = false;
    private bool _startedWebsocket = false;
    private bool _startingWebsocket = false;
    public bool disableWebsocket = false;
    private readonly string _baseURL = WebConfig.WebsocketBaseURL;
    private GameManager _gameManager;
    
    
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
        
        _httpRequestManager = new HttpRequestManager(this);
        _httpRequestManager.CreateGame();
        
        //Configure Websocket Connection
        Application.runInBackground = true; // Recommended for WebGL
    }

    async void Update()
    {
        if (!_gameIdReady && _httpRequestManager.HasGameId)
        {
            _gameManager.gameId = (int)_httpRequestManager.GameId;
            _gameIdReady = true;
        }

        if (!_startingWebsocket && _gameIdReady)
        {
            Debug.Log("Game ID: " + _gameManager.gameId);
            var url = $"{_baseURL}/game/{_gameManager.gameId}/ws/controls";
            Debug.Log("Connecting to: " + url);
            InitWebsocket(url);
            _startingWebsocket = true;
            await _websocket.Connect();
        }

        if (!_startedWebsocket && _gameIdReady)
        {
            if (_websocket.State == WebSocketState.Open)
            {
                _startedWebsocket = true;
                SendWebSocketMessage("Hi From Unity :)");
                GameReady = true;
            }
        }
    }

    void InitWebsocket(string url)
    {
        _websocket = new WebSocket(url);
        _websocket.OnOpen += () => Debug.Log("Connection open!");
        _websocket.OnError += (e) => Debug.Log("Error! " + e);
        _websocket.OnClose += (code) => Debug.Log("Connection closed!");
        _websocket.OnMessage += (bytes) =>
        {
            var message = System.Text.Encoding.UTF8.GetString(bytes);
            Debug.Log("Received: " + message);
        };
    }
    
        


    async void SendWebSocketMessage(string message)
    {
        if (_websocket.State == WebSocketState.Open)
        {
            await _websocket.SendText(message);
        }
        else
        {
            Debug.LogError("WebSocket is not open!");
        }
    }

    private async void OnApplicationQuit()
    {
        _httpRequestManager.StopGame();
        await _websocket.Close();
    }
    
}