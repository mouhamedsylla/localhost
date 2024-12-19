// Animation effects for the error page
export function initializeGlitchEffect(element) {
    const glitchEffect = () => {
        if (Math.random() < 0.1) {
            element.style.opacity = Math.random();
            setTimeout(() => {
                element.style.opacity = 1;
            }, 50);
        }
    };

    setInterval(glitchEffect, 100);
}

// Initialize all error page effects and animations
export function initializeErrorPage(errorCode) {
    // Set the error type attribute for styling
    document.body.setAttribute('data-error-type', errorCode);
    
    // Initialize glitch effect on error code
    const errorCodeElement = document.querySelector('.error-code');
    if (errorCodeElement) {
        initializeGlitchEffect(errorCodeElement);
    }
}