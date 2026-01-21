//! Stealth scripts for undetectable browser automation
//!
//! This module provides JavaScript injection scripts to make browser automation
//! undetectable by anti-bot systems. Based on Patchright's approach.

/// Get the stealth injection script
///
/// This script patches common automation detection vectors:
/// - navigator.webdriver
/// - chrome object
/// - plugins, languages, platform
/// - permissions
/// - WebGL vendor/renderer
/// - Canvas fingerprinting
/// - Window dimensions
pub fn get_stealth_script(
    webgl_spoof: bool,
    canvas_noise: bool,
    permissions_patch: bool,
) -> String {
    let mut script = String::new();

    // Navigator.webdriver patch
    script.push_str(
        r#"
// Patch navigator.webdriver
Object.defineProperty(navigator, 'webdriver', {
    get: () => false,
    configurable: true
});
"#,
    );

    // Chrome object stub
    script.push_str(
        r#"
// Stub chrome object if it doesn't exist
if (!window.chrome) {
    window.chrome = {};
}
if (!window.chrome.runtime) {
    window.chrome.runtime = {};
}
"#,
    );

    // Plugins patch
    script.push_str(
        r#"
// Patch plugins to look more natural
Object.defineProperty(navigator, 'plugins', {
    get: () => {
        const plugins = [
            {
                name: 'PDF Viewer',
                filename: 'internal-pdf-viewer',
                description: 'Portable Document Format'
            },
            {
                name: 'Chrome PDF Viewer',
                filename: 'internal-pdf-viewer',
                description: 'Portable Document Format'
            },
            {
                name: 'Chromium PDF Viewer',
                filename: 'internal-pdf-viewer',
                description: 'Portable Document Format'
            }
        ];
        return plugins;
    }
});
"#,
    );

    // Languages patch
    script.push_str(
        r#"
// Patch languages if empty
if (!navigator.languages || navigator.languages.length === 0) {
    Object.defineProperty(navigator, 'languages', {
        get: () => ['en-US', 'en'],
        configurable: true
    });
}
"#,
    );

    // Platform patch
    script.push_str(
        r#"
// Ensure platform is set
if (!navigator.platform) {
    Object.defineProperty(navigator, 'platform', {
        get: () => 'Win32',
        configurable: true
    });
}
"#,
    );

    // Permissions patch
    if permissions_patch {
        script.push_str(
            r#"
// Patch permissions.query
const originalQuery = navigator.permissions.query;
navigator.permissions.query = (parameters) => {
    return parameters.name === 'notifications'
        ? Promise.resolve({ state: Notification.permission })
        : originalQuery(parameters);
};
"#,
        );
    }

    // WebGL spoofing
    if webgl_spoof {
        script.push_str(
            r#"
// Spoof WebGL vendor and renderer
const getParameter = WebGLRenderingContext.prototype.getParameter;
WebGLRenderingContext.prototype.getParameter = function(parameter) {
    if (parameter === 37445) {
        return 'Intel Inc.';
    }
    if (parameter === 37446) {
        return 'Intel Iris OpenGL Engine';
    }
    return getParameter.apply(this, arguments);
};
"#,
        );
    }

    // Canvas noise
    if canvas_noise {
        script.push_str(
            r#"
// Add canvas noise for fingerprint randomization
const originalToDataURL = HTMLCanvasElement.prototype.toDataURL;
HTMLCanvasElement.prototype.toDataURL = function() {
    const context = this.getContext('2d');
    if (context) {
        const imageData = context.getImageData(0, 0, this.width, this.height);
        for (let i = 0; i < imageData.data.length; i += 4) {
            imageData.data[i] += Math.floor(Math.random() * 10) - 5;
            imageData.data[i + 1] += Math.floor(Math.random() * 10) - 5;
            imageData.data[i + 2] += Math.floor(Math.random() * 10) - 5;
        }
        context.putImageData(imageData, 0, 0);
    }
    return originalToDataURL.apply(this, arguments);
};
"#,
        );
    }

    // Window dimensions normalization
    script.push_str(
        r#"
// Normalize window dimensions (hairline fix)
if (window.outerWidth === 0) {
    Object.defineProperty(window, 'outerWidth', {
        get: () => window.innerWidth,
        configurable: true
    });
}
if (window.outerHeight === 0) {
    Object.defineProperty(window, 'outerHeight', {
        get: () => window.innerHeight,
        configurable: true
    });
}
"#,
    );

    script
}

/// Get a minimal stealth script (just navigator.webdriver)
pub fn get_minimal_stealth_script() -> &'static str {
    r#"
Object.defineProperty(navigator, 'webdriver', {
    get: () => false,
    configurable: true
});
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_stealth_script_basic() {
        let script = get_stealth_script(false, false, false);
        assert!(script.contains("navigator.webdriver"));
        assert!(script.contains("window.chrome"));
        assert!(!script.contains("WebGL"));
        assert!(!script.contains("canvas"));
    }

    #[test]
    fn test_get_stealth_script_full() {
        let script = get_stealth_script(true, true, true);
        assert!(script.contains("navigator.webdriver"));
        assert!(script.contains("WebGL"));
        assert!(script.contains("canvas"));
        assert!(script.contains("permissions"));
    }

    #[test]
    fn test_minimal_stealth() {
        let script = get_minimal_stealth_script();
        assert!(script.contains("navigator.webdriver"));
    }
}
