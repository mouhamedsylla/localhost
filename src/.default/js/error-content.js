export function setErrorContent(config) {
    // Use the injected message if available
    const errorMessage = window.ERROR_MESSAGE || config.message;
    document.querySelector('.error-code').textContent = config.code;
    document.querySelector('.error-message').textContent = errorMessage;
    document.querySelector('.back-button').textContent = config.buttonText || 'Go Back';
}