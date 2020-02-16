using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class GlobalStateScript : MonoBehaviour
{
    private PFCanvasFontContext fontContext;

    [SerializeField]
    bool isPathfinderEnabled = false;

    // Start is called before the first frame update
    void Start()
    {
        fontContext = new PFCanvasFontContext();
    }

    
    public bool IsPathfinderEnabled() {
        isPathfinderEnabled = (fontContext != null);
        return fontContext != null;
    }

    public void TogglePathfinderEnabled() {
        if (fontContext != null) {
            fontContext = null;
            PFPlugin.ReleaseResources();
        } else {
            fontContext = new PFCanvasFontContext();
        }
    }

    public PFCanvasFontContext GetFontContext() {
        return fontContext;
    }
}
