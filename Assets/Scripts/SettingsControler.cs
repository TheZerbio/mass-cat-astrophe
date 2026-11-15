using UnityEngine;
using UnityEngine.SceneManagement;

public class SettingsControler : MonoBehaviour
{
    private GameManager gameManager;
    // Start is called once before the first execution of Update after the MonoBehaviour is created
    void Start()
    {
        gameManager = GameObject.Find("PersistantGameObjects").GetComponent<GameManager>();
    }

    // Update is called once per frame
    void Update()
    {
        
    }

    public void SetVolume(float volume)
    {
        gameManager.Sounds.Volume = volume;
    }
    
    public void SetCatVolume(float volume)
    {
        gameManager.Sounds.CatVolume = volume;
        gameManager.Sounds.CatSoundManager.PlayMeow();
    }
    
    public void SetSfxVolume(float volume)
    {
        gameManager.Sounds.SfxVolume = volume;
        gameManager.Sounds.SfxManager.PlaySwoosh();
    }
    
    public void SetMusicVolume(float volume)
    {
        gameManager.Sounds.MusicVolume = volume;
    }

    public void LoadMainMenu()
    {
        SceneManager.LoadScene("MainMenu");
    }
}
