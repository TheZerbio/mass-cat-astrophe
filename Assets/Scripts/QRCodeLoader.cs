using UnityEngine;
using UnityEngine.SceneManagement;
using UnityEngine.UI;

public class QRCodeLoader : MonoBehaviour
{
    private GameManager gameManager;
    public Image QRCodeImage;
    public GameObject PlayButton;
    public GameObject QuitButton;

    // Start is called once before the first execution of Update after the MonoBehaviour is created
    void Start()
    {
        gameManager = GameObject.Find("PersistantGameObjects").GetComponent<GameManager>();
#if UNITY_WEBGL && !UNITY_EDITOR
        // a browser tab can't be closed by the game, Application.Quit() does nothing there
        if (QuitButton != null) QuitButton.SetActive(false);
#endif
    }

    public void ShowQrCode()
    {
        Texture2D qrCodeTexture = null;
        while (qrCodeTexture == null)
        {
            qrCodeTexture = gameManager.GenerateJoinQrCode(2);
        }
        QRCodeImage.sprite = Sprite.Create(qrCodeTexture, new Rect(0, 0, qrCodeTexture.width, qrCodeTexture.height), new Vector2(0.5f, 0.5f));
        QRCodeImage.color = new Color(1, 1, 1, 1);
        PlayButton?.SetActive(true);
    }

    public void LoadLevel()
    {
        gameManager.Network.StartGame();
        SceneManager.LoadScene("nics test and play scene");
    }

    public void LoadSettings()
    {
        SceneManager.LoadScene("Settings");
    }

    public void Quit()
    {
        Application.Quit();
    }
}
