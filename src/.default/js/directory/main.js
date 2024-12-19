import { initializeDirectory } from './initialize.js';
import { setupEventListeners } from './events.js';

document.addEventListener('DOMContentLoaded', () => {
    initializeDirectory();
    setupEventListeners();
});