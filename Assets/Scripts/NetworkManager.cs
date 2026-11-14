using UnityEngine;
using NativeWebSocket;


public class NetworkManager : MonoBehaviour
{
    private GUID guid;
    WebSocket websocket;
    
    async void Start()
    {
        Application.runInBackground = true; // Recommended for WebGL

        websocket = new WebSocket("ws://localhost:3000");

        websocket.OnOpen += () => Debug.Log("Connection open!");
        websocket.OnError += (e) => Debug.Log("Error! " + e);
        websocket.OnClose += (code) => Debug.Log("Connection closed!");

        websocket.OnMessage += (bytes) =>
        {
            var message = System.Text.Encoding.UTF8.GetString(bytes);
            Debug.Log("Received: " + message);
        };

        await websocket.Connect();
    }

    async void SendWebSocketMessage(string message)
    {
        if (websocket.State == WebSocketState.Open)
        {
            await websocket.SendText(message);
        }
        else
        {
            Debug.LogError("WebSocket is not open!");
        }
    }

    private async void OnApplicationQuit()
    {
        await websocket.Close();
    }
}