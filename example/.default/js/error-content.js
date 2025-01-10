// Update page content with error information
export function setErrorContent(config) {
    // Update page title
    document.title = `${config.title} - ${config.code}`;
    
    // Update error code display
    const errorCodeElement = document.querySelector('.error-code');
    if (errorCodeElement) {
        errorCodeElement.textContent = config.code;
    }
    
    // Update error message
    const errorMessageElement = document.querySelector('.error-message');
    if (errorMessageElement) {
        errorMessageElement.textContent = config.message;
    }
    
    // Update button text
    const backButtonElement = document.querySelector('.back-button');
    if (backButtonElement) {
        backButtonElement.textContent = config.buttonText;
    }
}