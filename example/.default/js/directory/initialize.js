import { renderDirectoryContent } from './render.js';
import { directoryData } from './data.js';
import { updateStats } from './stats.js';

export async function initializeDirectory() {
    try {
        const path = window.location.pathname;
        const content = directoryData[path];
        
        if (!content) {
            throw new Error('Directory not found');
        }
        
        updateCurrentPath(path);
        renderDirectoryContent(content);
        updateStats(content);
    } catch (error) {
        window.location.href = 'error-template.html?code=404';
    }
}

function updateCurrentPath(path) {
    const pathElement = document.querySelector('.current-path');
    if (pathElement) {
        pathElement.textContent = `Location: ${path}`;
    }
}